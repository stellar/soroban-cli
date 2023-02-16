package ingest

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/stellar/go/historyarchive"
	"github.com/stellar/go/ingest"
	backends "github.com/stellar/go/ingest/ledgerbackend"
	"github.com/stellar/go/support/log"
	"github.com/stellar/go/xdr"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/db"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/events"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/transactions"
)

const (
	ledgerEntryBaselineProgressLogPeriod = 10000
)

type Config struct {
	Logger            *log.Entry
	DB                db.ReadWriter
	EventStore        *events.MemoryStore
	TransactionStore  *transactions.MemoryStore
	NetworkPassPhrase string
	Archive           historyarchive.ArchiveInterface
	LedgerBackend     backends.LedgerBackend
	Timeout           time.Duration
}

func NewService(cfg Config) (*Service, error) {
	ctx, done := context.WithCancel(context.Background())
	o := Service{
		logger:            cfg.Logger,
		db:                cfg.DB,
		eventStore:        cfg.EventStore,
		transactionStore:  cfg.TransactionStore,
		ledgerBackend:     cfg.LedgerBackend,
		networkPassPhrase: cfg.NetworkPassPhrase,
		timeout:           cfg.Timeout,
		done:              done,
	}
	o.wg.Add(1)
	go func() {
		err := o.run(ctx, cfg.Archive)
		if err != nil && !errors.Is(err, context.Canceled) {
			o.logger.WithError(err).Fatal("could not run ingestion")
		}
	}()
	return &o, nil
}

type Service struct {
	logger            *log.Entry
	db                db.ReadWriter
	eventStore        *events.MemoryStore
	transactionStore  *transactions.MemoryStore
	ledgerBackend     backends.LedgerBackend
	timeout           time.Duration
	networkPassPhrase string
	done              context.CancelFunc
	wg                sync.WaitGroup
}

func (s *Service) Close() error {
	s.done()
	s.wg.Wait()
	return nil
}

func (s *Service) run(ctx context.Context, archive historyarchive.ArchiveInterface) error {
	defer s.wg.Done()
	// Create a ledger-entry baseline from a checkpoint if it wasn't done before
	// (after that we will be adding deltas from txmeta ledger entry changes)
	nextLedgerSeq, checkPointFillErr, err := s.maybeFillEntriesFromCheckpoint(ctx, archive)
	if err != nil {
		return err
	}

	prepareRangeCtx, cancelPrepareRange := context.WithTimeout(ctx, s.timeout)
	if err := s.ledgerBackend.PrepareRange(prepareRangeCtx, backends.UnboundedRange(nextLedgerSeq)); err != nil {
		cancelPrepareRange()
		return err
	}
	cancelPrepareRange()

	// Make sure that the checkpoint prefill (if any), happened before starting to apply deltas
	if err := <-checkPointFillErr; err != nil {
		return err
	}

	for ; ; nextLedgerSeq++ {
		if err := s.ingest(ctx, nextLedgerSeq); err != nil {
			return err
		}
	}
}

func (s *Service) maybeFillEntriesFromCheckpoint(ctx context.Context, archive historyarchive.ArchiveInterface) (uint32, chan error, error) {
	checkPointFillErr := make(chan error, 1)
	// Skip creating a ledger-entry baseline if the DB was initialized
	curLedgerSeq, err := s.db.GetLatestLedgerSequence(ctx)
	if err == db.ErrEmptyDB {
		var checkpointLedger uint32
		if root, rootErr := archive.GetRootHAS(); rootErr != nil {
			return 0, checkPointFillErr, rootErr
		} else {
			checkpointLedger = root.CurrentLedger
		}

		// DB is empty, let's fill it from the History Archive, using the latest available checkpoint
		// Do it in parallel with the upcoming captive core preparation to save time
		s.logger.Infof("Found an empty database, creating ledger-entry baseline from the most recent checkpoint (%d). This can take up to 30 minutes, depending on the network", checkpointLedger)
		go func() {
			checkPointFillErr <- s.fillEntriesFromCheckpoint(ctx, archive, checkpointLedger)
		}()
		return checkpointLedger + 1, checkPointFillErr, nil
	} else if err != nil {
		return 0, checkPointFillErr, err
	} else {
		checkPointFillErr <- nil
		return curLedgerSeq + 1, checkPointFillErr, nil
	}
}

func (s *Service) fillEntriesFromCheckpoint(ctx context.Context, archive historyarchive.ArchiveInterface, checkpointLedger uint32) error {
	checkpointCtx, cancelCheckpointCtx := context.WithTimeout(ctx, s.timeout)
	defer cancelCheckpointCtx()

	reader, err := ingest.NewCheckpointChangeReader(checkpointCtx, archive, checkpointLedger)
	if err != nil {
		return err
	}

	tx, err := s.db.NewTx(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if err := tx.Rollback(); err != nil {
			s.logger.WithError(err).Warn("could not rollback fillEntriesFromCheckpoint write transactions")
		}
	}()

	if err := s.ingestLedgerEntryChanges(ctx, reader, tx, ledgerEntryBaselineProgressLogPeriod); err != nil {
		return err
	}
	if err := reader.Close(); err != nil {
		return err
	}

	s.logger.Info("Committing checkpoint ledger entries")
	if err := tx.Commit(checkpointLedger); err != nil {
		return err
	}
	s.logger.Info("Finished checkpoint processing")
	return nil
}

func (s *Service) ingest(ctx context.Context, sequence uint32) error {
	s.logger.Infof("Applying txmeta for ledger %d", sequence)
	ledgerCloseMeta, err := s.ledgerBackend.GetLedger(ctx, sequence)
	if err != nil {
		return err
	}
	reader, err := ingest.NewLedgerChangeReaderFromLedgerCloseMeta(s.networkPassPhrase, ledgerCloseMeta)
	if err != nil {
		return err
	}
	tx, err := s.db.NewTx(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if err := tx.Rollback(); err != nil {
			s.logger.WithError(err).Warn("could not rollback ingest write transactions")
		}
	}()

	if err := s.ingestLedgerEntryChanges(ctx, reader, tx, 0); err != nil {
		return err
	}
	if err := reader.Close(); err != nil {
		return err
	}

	if err := s.ingestLedgerCloseMeta(tx, ledgerCloseMeta); err != nil {
		return err
	}

	if err := tx.Commit(sequence); err != nil {
		return err
	}
	return nil
}

func (s *Service) ingestLedgerCloseMeta(tx db.WriteTx, ledgerCloseMeta xdr.LedgerCloseMeta) error {
	if err := tx.LedgerWriter().InsertLedger(ledgerCloseMeta); err != nil {
		return err
	}

	if err := s.eventStore.IngestEvents(ledgerCloseMeta); err != nil {
		return err
	}

	if err := s.transactionStore.IngestTransactions(ledgerCloseMeta); err != nil {
		return err
	}
	return nil
}
