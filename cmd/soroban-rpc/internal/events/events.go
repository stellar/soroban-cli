package events

import (
	"errors"
	"io"
	"sort"
	"sync"

	"github.com/stellar/go/ingest"
	"github.com/stellar/go/xdr"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/ledgerbucketwindow"
)

type bucket struct {
	ledgerSeq            uint32
	ledgerCloseTimestamp int64
	events               []event
}

type event struct {
	contents   xdr.ContractEvent
	txIndex    uint32
	opIndex    uint32
	eventIndex uint32
}

func (e event) cursor(ledgerSeq uint32) Cursor {
	return Cursor{
		Ledger: ledgerSeq,
		Tx:     e.txIndex,
		Op:     e.opIndex,
		Event:  e.eventIndex,
	}
}

// MemoryStore is an in-memory store of soroban events.
type MemoryStore struct {
	// networkPassphrase is an immutable string containing the
	// Stellar network passphrase.
	// Accessing networkPassphrase does not need to be protected
	// by the lock
	networkPassphrase string
	// lock protects the mutable fields below
	lock           sync.RWMutex
	eventsByLedger *ledgerbucketwindow.LedgerBucketWindow[[]event]
}

// NewMemoryStore creates a new MemoryStore.
// The retention window is in units of ledgers.
// All events occurring in the following ledger range
// [ latestLedger - retentionWindow, latestLedger ]
// will be included in the MemoryStore. If the MemoryStore
// is full, any events from new ledgers will evict
// older entries outside the retention window.
func NewMemoryStore(networkPassphrase string, retentionWindow uint32) *MemoryStore {
	window := ledgerbucketwindow.NewLedgerBucketWindow[[]event](retentionWindow)
	return &MemoryStore{
		networkPassphrase: networkPassphrase,
		eventsByLedger:    window,
	}
}

// Range defines a [Start, End) interval of Soroban events.
type Range struct {
	// Start defines the (inclusive) start of the range.
	Start Cursor
	// ClampStart indicates whether Start should be clamped up
	// to the earliest ledger available if Start is too low.
	ClampStart bool
	// End defines the (exclusive) end of the range.
	End Cursor
	// ClampEnd indicates whether End should be clamped down
	// to the latest ledger available if End is too high.
	ClampEnd bool
}

// Scan applies f on all the events occurring in the given range.
// The events are processed in sorted ascending Cursor order.
// If f returns false, the scan terminates early (f will not be applied on
// remaining events in the range). Note that a read lock is held for the
// entire duration of the Scan function so f should be written in a way
// to minimize latency.
func (m *MemoryStore) Scan(eventRange Range, f func(xdr.ContractEvent, Cursor, int64) bool) (uint32, error) {
	m.lock.RLock()
	defer m.lock.RUnlock()

	if err := m.validateRange(&eventRange); err != nil {
		return 0, err
	}

	firstLedgerInRange := eventRange.Start.Ledger
	firstLedgerInWindow := m.eventsByLedger.Get(0).LedgerSeq
	lastLedgerInWindow := firstLedgerInWindow + (m.eventsByLedger.Len() - 1)
	for i := firstLedgerInRange - firstLedgerInWindow; i < m.eventsByLedger.Len(); i++ {
		bucket := m.eventsByLedger.Get(i)
		events := bucket.BucketContent
		if bucket.LedgerSeq == firstLedgerInRange {
			// we need to seek for the beginning of the events in the first bucket in the range
			events = seek(events, eventRange.Start)
		}
		timestamp := bucket.LedgerCloseTimestamp
		for _, event := range events {
			cur := event.cursor(bucket.LedgerSeq)
			if eventRange.End.Cmp(cur) <= 0 {
				return lastLedgerInWindow, nil
			}
			if !f(event.contents, cur, timestamp) {
				return lastLedgerInWindow, nil
			}
		}
	}
	return lastLedgerInWindow, nil
}

// validateRange checks if the range falls within the bounds
// of the events in the memory store.
// validateRange should be called with the read lock.
func (m *MemoryStore) validateRange(eventRange *Range) error {
	if m.eventsByLedger.Len() == 0 {
		return errors.New("event store is empty")
	}
	firstBucket := m.eventsByLedger.Get(0)
	min := Cursor{Ledger: firstBucket.LedgerSeq}
	if eventRange.Start.Cmp(min) < 0 {
		if eventRange.ClampStart {
			eventRange.Start = min
		} else {
			return errors.New("start is before oldest ledger")
		}
	}
	max := Cursor{Ledger: min.Ledger + m.eventsByLedger.Len()}
	if eventRange.Start.Cmp(max) >= 0 {
		return errors.New("start is after newest ledger")
	}
	if eventRange.End.Cmp(max) > 0 {
		if eventRange.ClampEnd {
			eventRange.End = max
		} else {
			return errors.New("end is after latest ledger")
		}
	}

	if eventRange.Start.Cmp(eventRange.End) >= 0 {
		return errors.New("start is not before end")
	}

	return nil
}

// seek returns the subset of all events which occur
// at a point greater than or equal to the given cursor.
// events must be sorted in ascending order.
func seek(events []event, cursor Cursor) []event {
	j := sort.Search(len(events), func(i int) bool {
		return cursor.Cmp(events[i].cursor(cursor.Ledger)) <= 0
	})
	return events[j:]
}

// IngestEvents adds new events from the given ledger into the store.
// As a side effect, events which fall outside the retention window are
// removed from the store.
func (m *MemoryStore) IngestEvents(ledgerCloseMeta xdr.LedgerCloseMeta) error {
	// no need to acquire the lock because the networkPassphrase field
	// is immutable
	events, err := readEvents(m.networkPassphrase, ledgerCloseMeta)
	if err != nil {
		return err
	}
	bucket := ledgerbucketwindow.LedgerBucket[[]event]{
		LedgerSeq:            ledgerCloseMeta.LedgerSequence(),
		LedgerCloseTimestamp: int64(ledgerCloseMeta.LedgerHeaderHistoryEntry().Header.ScpValue.CloseTime),
		BucketContent:        events,
	}
	m.lock.Lock()
	m.eventsByLedger.Append(bucket)
	m.lock.Unlock()
	return err
}

func readEvents(networkPassphrase string, ledgerCloseMeta xdr.LedgerCloseMeta) (events []event, err error) {
	var txReader *ingest.LedgerTransactionReader
	txReader, err = ingest.NewLedgerTransactionReaderFromLedgerCloseMeta(networkPassphrase, ledgerCloseMeta)
	if err != nil {
		return
	}
	defer func() {
		closeErr := txReader.Close()
		if err == nil {
			err = closeErr
		}
	}()

	for {
		var tx ingest.LedgerTransaction
		tx, err = txReader.Read()
		if err == io.EOF {
			err = nil
			break
		}
		if err != nil {
			return
		}

		if !tx.Result.Successful() {
			continue
		}
		for i := range tx.Envelope.Operations() {
			opIndex := uint32(i)
			var opEvents []xdr.ContractEvent
			opEvents, err = tx.GetOperationEvents(opIndex)
			if err != nil {
				return
			}
			for eventIndex, opEvent := range opEvents {
				events = append(events, event{
					contents:   opEvent,
					txIndex:    tx.Index,
					opIndex:    opIndex,
					eventIndex: uint32(eventIndex),
				})
			}
		}
	}
	return events, err
}
