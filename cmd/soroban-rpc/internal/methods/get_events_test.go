package methods

import (
	"encoding/json"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/stellar/go/keypair"
	"github.com/stellar/go/network"
	"github.com/stellar/go/xdr"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/events"
)

func TestTopicFilterMatches(t *testing.T) {
	transferSym := xdr.ScSymbol("transfer")
	transfer := xdr.ScVal{
		Type: xdr.ScValTypeScvSymbol,
		Sym:  &transferSym,
	}
	sixtyfour := xdr.Int64(64)
	number := xdr.ScVal{
		Type: xdr.ScValTypeScvU63,
		U63:  &sixtyfour,
	}
	star := "*"
	for _, tc := range []struct {
		name     string
		filter   TopicFilter
		includes []xdr.ScVec
		excludes []xdr.ScVec
	}{
		{
			name:   "<empty>",
			filter: nil,
			includes: []xdr.ScVec{
				{},
			},
			excludes: []xdr.ScVec{
				{transfer},
			},
		},

		// Exact matching
		{
			name: "ScSymbol(transfer)",
			filter: []SegmentFilter{
				{scval: &transfer},
			},
			includes: []xdr.ScVec{
				{transfer},
			},
			excludes: []xdr.ScVec{
				{number},
				{transfer, transfer},
			},
		},

		// Star
		{
			name: "*",
			filter: []SegmentFilter{
				{wildcard: &star},
			},
			includes: []xdr.ScVec{
				{transfer},
			},
			excludes: []xdr.ScVec{
				{transfer, transfer},
			},
		},
		{
			name: "*/transfer",
			filter: []SegmentFilter{
				{wildcard: &star},
				{scval: &transfer},
			},
			includes: []xdr.ScVec{
				{number, transfer},
				{transfer, transfer},
			},
			excludes: []xdr.ScVec{
				{number},
				{number, number},
				{number, transfer, number},
				{transfer},
				{transfer, number},
				{transfer, transfer, transfer},
			},
		},
		{
			name: "transfer/*",
			filter: []SegmentFilter{
				{scval: &transfer},
				{wildcard: &star},
			},
			includes: []xdr.ScVec{
				{transfer, number},
				{transfer, transfer},
			},
			excludes: []xdr.ScVec{
				{number},
				{number, number},
				{number, transfer, number},
				{transfer},
				{number, transfer},
				{transfer, transfer, transfer},
			},
		},
		{
			name: "transfer/*/*",
			filter: []SegmentFilter{
				{scval: &transfer},
				{wildcard: &star},
				{wildcard: &star},
			},
			includes: []xdr.ScVec{
				{transfer, number, number},
				{transfer, transfer, transfer},
			},
			excludes: []xdr.ScVec{
				{number},
				{number, number},
				{number, transfer},
				{number, transfer, number, number},
				{transfer},
				{transfer, transfer, transfer, transfer},
			},
		},
		{
			name: "transfer/*/number",
			filter: []SegmentFilter{
				{scval: &transfer},
				{wildcard: &star},
				{scval: &number},
			},
			includes: []xdr.ScVec{
				{transfer, number, number},
				{transfer, transfer, number},
			},
			excludes: []xdr.ScVec{
				{number},
				{number, number},
				{number, number, number},
				{number, transfer, number},
				{transfer},
				{number, transfer},
				{transfer, transfer, transfer},
				{transfer, number, transfer},
			},
		},
	} {
		name := tc.name
		if name == "" {
			name = topicFilterToString(tc.filter)
		}
		t.Run(name, func(t *testing.T) {
			for _, include := range tc.includes {
				assert.True(
					t,
					tc.filter.Matches(include),
					"Expected %v filter to include %v",
					name,
					include,
				)
			}
			for _, exclude := range tc.excludes {
				assert.False(
					t,
					tc.filter.Matches(exclude),
					"Expected %v filter to exclude %v",
					name,
					exclude,
				)
			}
		})
	}
}

func TestTopicFilterJSON(t *testing.T) {
	var got TopicFilter

	assert.NoError(t, json.Unmarshal([]byte("[]"), &got))
	assert.Equal(t, TopicFilter{}, got)

	star := "*"
	assert.NoError(t, json.Unmarshal([]byte("[\"*\"]"), &got))
	assert.Equal(t, TopicFilter{{wildcard: &star}}, got)

	sixtyfour := xdr.Int64(64)
	scval := xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &sixtyfour}
	scvalstr, err := xdr.MarshalBase64(scval)
	assert.NoError(t, err)
	assert.NoError(t, json.Unmarshal([]byte(fmt.Sprintf("[%q]", scvalstr)), &got))
	assert.Equal(t, TopicFilter{{scval: &scval}}, got)
}

func topicFilterToString(t TopicFilter) string {
	var s []string
	for _, segment := range t {
		if segment.wildcard != nil {
			s = append(s, *segment.wildcard)
		} else if segment.scval != nil {
			out, err := xdr.MarshalBase64(*segment.scval)
			if err != nil {
				panic(err)
			}
			s = append(s, out)
		} else {
			panic("Invalid topic filter")
		}
	}
	if len(s) == 0 {
		s = append(s, "<empty>")
	}
	return strings.Join(s, "/")
}

func TestGetEventsRequestValid(t *testing.T) {
	assert.NoError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters:     []EventFilter{},
		Pagination:  nil,
	}).Valid(1000))

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters:     []EventFilter{},
		Pagination:  &PaginationOptions{Limit: 1001},
	}).Valid(1000), "limit must not exceed 1000")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 0,
		Filters:     []EventFilter{},
		Pagination:  nil,
	}).Valid(1000), "startLedger must be positive")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: -100,
		Filters:     []EventFilter{},
		Pagination:  nil,
	}).Valid(1000), "startLedger must be positive")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{}, {}, {}, {}, {}, {},
		},
		Pagination: nil,
	}).Valid(1000), "maximum 5 filters per request")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{EventType: "foo"},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: if set, type must be either 'system' or 'contract'")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{ContractIDs: []string{
				"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
				"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
				"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
				"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
				"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
				"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			}},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: maximum 5 contract IDs per filter")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{ContractIDs: []string{"a"}},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: contract ID 1 invalid")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{
				Topics: []TopicFilter{
					{}, {}, {}, {}, {}, {},
				},
			},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: maximum 5 topics per filter")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{Topics: []TopicFilter{
				{},
			}},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: topic 1 invalid: topic must have at least one segment")

	assert.EqualError(t, (&GetEventsRequest{
		StartLedger: 1,
		Filters: []EventFilter{
			{Topics: []TopicFilter{
				{
					{},
					{},
					{},
					{},
					{},
				},
			}},
		},
		Pagination: nil,
	}).Valid(1000), "filter 1 invalid: topic 1 invalid: topic cannot have more than 4 segments")
}

func TestGetEvents(t *testing.T) {
	now := time.Now().UTC()
	counter := xdr.ScSymbol("COUNTER")
	counterScVal := xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter}
	counterXdr, err := xdr.MarshalBase64(counterScVal)
	assert.NoError(t, err)

	t.Run("empty", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		_, err = handler.getEvents(GetEventsRequest{
			StartLedger: 1,
		})
		assert.EqualError(t, err, "[-32600] event store is empty")
	})

	t.Run("startLedger validation", func(t *testing.T) {
		contractID := xdr.Hash([32]byte{})
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		var txMeta []xdr.TransactionMeta
		txMeta = append(txMeta, transactionMetaWithEvents(
			[]xdr.ContractEvent{
				contractEvent(
					contractID,
					xdr.ScVec{xdr.ScVal{
						Type: xdr.ScValTypeScvSymbol,
						Sym:  &counter,
					}},
					xdr.ScVal{
						Type: xdr.ScValTypeScvSymbol,
						Sym:  &counter,
					},
				),
			},
		))
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(2, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		_, err = handler.getEvents(GetEventsRequest{
			StartLedger: 1,
		})
		assert.EqualError(t, err, "[-32600] start is before oldest ledger")

		_, err = handler.getEvents(GetEventsRequest{
			StartLedger: 3,
		})
		assert.EqualError(t, err, "[-32600] start is after newest ledger")
	})

	t.Run("no filtering returns all", func(t *testing.T) {
		contractID := xdr.Hash([32]byte{})
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		var txMeta []xdr.TransactionMeta
		for i := 0; i < 10; i++ {
			txMeta = append(txMeta, transactionMetaWithEvents(
				[]xdr.ContractEvent{
					contractEvent(
						contractID,
						xdr.ScVec{xdr.ScVal{
							Type: xdr.ScValTypeScvSymbol,
							Sym:  &counter,
						}},
						xdr.ScVal{
							Type: xdr.ScValTypeScvSymbol,
							Sym:  &counter,
						},
					),
				},
			))
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
		})
		assert.NoError(t, err)

		var expected []EventInfo
		for i := range txMeta {
			id := events.Cursor{
				Ledger: 1,
				Tx:     uint32(i + 1),
				Op:     0,
				Event:  0,
			}.String()
			value, err := xdr.MarshalBase64(xdr.ScVal{
				Type: xdr.ScValTypeScvSymbol,
				Sym:  &counter,
			})
			assert.NoError(t, err)
			expected = append(expected, EventInfo{
				EventType:      EventTypeContract,
				Ledger:         1,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     "0000000000000000000000000000000000000000000000000000000000000000",
				ID:             id,
				PagingToken:    id,
				Topic:          []string{value},
				Value: EventInfoValue{
					XDR: value,
				},
			})
		}
		assert.Equal(t, expected, results)
	})

	t.Run("filtering by contract id", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		var txMeta []xdr.TransactionMeta
		contractIds := []xdr.Hash{
			xdr.Hash([32]byte{}),
			xdr.Hash([32]byte{1}),
		}
		for i := 0; i < 5; i++ {
			txMeta = append(txMeta, transactionMetaWithEvents(
				[]xdr.ContractEvent{
					contractEvent(
						contractIds[i%len(contractIds)],
						xdr.ScVec{xdr.ScVal{
							Type: xdr.ScValTypeScvSymbol,
							Sym:  &counter,
						}},
						xdr.ScVal{
							Type: xdr.ScValTypeScvSymbol,
							Sym:  &counter,
						},
					),
				},
			))
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Filters: []EventFilter{
				{ContractIDs: []string{contractIds[0].HexString()}},
			},
		})
		assert.NoError(t, err)

		expectedIds := []string{
			events.Cursor{Ledger: 1, Tx: 1, Op: 0, Event: 0}.String(),
			events.Cursor{Ledger: 1, Tx: 3, Op: 0, Event: 0}.String(),
			events.Cursor{Ledger: 1, Tx: 5, Op: 0, Event: 0}.String(),
		}
		eventIds := []string{}
		for _, event := range results {
			eventIds = append(eventIds, event.ID)
		}
		assert.Equal(t, expectedIds, eventIds)
	})

	t.Run("filtering by topic", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		var txMeta []xdr.TransactionMeta
		contractID := xdr.Hash([32]byte{})
		for i := 0; i < 10; i++ {
			number := xdr.Int64(i)
			txMeta = append(txMeta, transactionMetaWithEvents(
				[]xdr.ContractEvent{
					// Generate a unique topic like /counter/4 for each event so we can check
					contractEvent(
						contractID,
						xdr.ScVec{
							xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
							xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
						},
						xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
					),
				},
			))
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		number := xdr.Int64(4)
		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Filters: []EventFilter{
				{Topics: []TopicFilter{
					[]SegmentFilter{
						{scval: &xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter}},
						{scval: &xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number}},
					},
				}},
			},
		})
		assert.NoError(t, err)

		id := events.Cursor{Ledger: 1, Tx: 5, Op: 0, Event: 0}.String()
		assert.NoError(t, err)
		value, err := xdr.MarshalBase64(xdr.ScVal{
			Type: xdr.ScValTypeScvU63,
			U63:  &number,
		})
		assert.NoError(t, err)
		expected := []EventInfo{
			{
				EventType:      EventTypeContract,
				Ledger:         1,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     "0000000000000000000000000000000000000000000000000000000000000000",
				ID:             id,
				PagingToken:    id,
				Topic:          []string{counterXdr, value},
				Value:          EventInfoValue{XDR: value},
			},
		}
		assert.Equal(t, expected, results)
	})

	t.Run("filtering by both contract id and topic", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		contractID := xdr.Hash([32]byte{})
		otherContractID := xdr.Hash([32]byte{1})
		number := xdr.Int64(1)
		txMeta := []xdr.TransactionMeta{
			// This matches neither the contract id nor the topic
			transactionMetaWithEvents([]xdr.ContractEvent{
				contractEvent(
					otherContractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
				),
			}),
			// This matches the contract id but not the topic
			transactionMetaWithEvents([]xdr.ContractEvent{
				contractEvent(
					contractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
				),
			}),
			// This matches the topic but not the contract id
			transactionMetaWithEvents([]xdr.ContractEvent{
				contractEvent(
					otherContractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
						xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
				),
			}),
			// This matches both the contract id and the topic
			transactionMetaWithEvents([]xdr.ContractEvent{
				contractEvent(
					contractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
						xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
				),
			}),
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Filters: []EventFilter{
				{
					ContractIDs: []string{contractID.HexString()},
					Topics: []TopicFilter{
						[]SegmentFilter{
							{scval: &xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter}},
							{scval: &xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number}},
						},
					},
				},
			},
		})
		assert.NoError(t, err)

		id := events.Cursor{Ledger: 1, Tx: 4, Op: 0, Event: 0}.String()
		value, err := xdr.MarshalBase64(xdr.ScVal{
			Type: xdr.ScValTypeScvU63,
			U63:  &number,
		})
		assert.NoError(t, err)
		expected := []EventInfo{
			{
				EventType:      EventTypeContract,
				Ledger:         1,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     contractID.HexString(),
				ID:             id,
				PagingToken:    id,
				Topic:          []string{counterXdr, value},
				Value:          EventInfoValue{XDR: value},
			},
		}
		assert.Equal(t, expected, results)
	})

	t.Run("filtering by event type", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		contractID := xdr.Hash([32]byte{})
		txMeta := []xdr.TransactionMeta{
			transactionMetaWithEvents([]xdr.ContractEvent{
				contractEvent(
					contractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
				),
				systemEvent(
					contractID,
					xdr.ScVec{
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
					},
					xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &counter},
				),
			}),
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Filters: []EventFilter{
				{EventType: EventTypeSystem},
			},
		})
		assert.NoError(t, err)

		id := events.Cursor{Ledger: 1, Tx: 1, Op: 0, Event: 1}.String()
		expected := []EventInfo{
			{
				EventType:      EventTypeSystem,
				Ledger:         1,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     contractID.HexString(),
				ID:             id,
				PagingToken:    id,
				Topic:          []string{counterXdr},
				Value:          EventInfoValue{XDR: counterXdr},
			},
		}
		assert.Equal(t, expected, results)
	})

	t.Run("with limit", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		contractID := xdr.Hash([32]byte{})
		var txMeta []xdr.TransactionMeta
		for i := 0; i < 180; i++ {
			number := xdr.Int64(i)
			txMeta = append(txMeta, transactionMetaWithEvents(
				[]xdr.ContractEvent{
					contractEvent(
						contractID,
						xdr.ScVec{
							xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
						},
						xdr.ScVal{Type: xdr.ScValTypeScvU63, U63: &number},
					),
				},
			))
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(1, now.Unix(), txMeta...)))

		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Filters:     []EventFilter{},
			Pagination:  &PaginationOptions{Limit: 10},
		})
		assert.NoError(t, err)

		var expected []EventInfo
		for i := 0; i < 10; i++ {
			id := events.Cursor{
				Ledger: 1,
				Tx:     uint32(i + 1),
				Op:     0,
				Event:  0,
			}.String()
			value, err := xdr.MarshalBase64(txMeta[i].MustV3().Events[0].Events[0].Body.MustV0().Data)
			assert.NoError(t, err)
			expected = append(expected, EventInfo{
				EventType:      EventTypeContract,
				Ledger:         1,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     "0000000000000000000000000000000000000000000000000000000000000000",
				ID:             id,
				PagingToken:    id,
				Topic:          []string{value},
				Value: EventInfoValue{
					XDR: value,
				},
			})
		}
		assert.Equal(t, expected, results)
	})

	t.Run("with cursor", func(t *testing.T) {
		store, err := events.NewMemoryStore("unit-tests", 100)
		assert.NoError(t, err)
		contractID := xdr.Hash([32]byte{})
		datas := []xdr.ScSymbol{
			// ledger/transaction/operation/event
			xdr.ScSymbol("5/1/0/0"),
			xdr.ScSymbol("5/1/0/1"),
			xdr.ScSymbol("5/1/1/0"),
			xdr.ScSymbol("5/1/1/1"),
		}
		txMeta := []xdr.TransactionMeta{
			transactionMetaWithEvents(
				[]xdr.ContractEvent{
					contractEvent(
						contractID,
						xdr.ScVec{
							counterScVal,
						},
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &datas[0]},
					),
					contractEvent(
						contractID,
						xdr.ScVec{
							counterScVal,
						},
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &datas[1]},
					),
				},
				[]xdr.ContractEvent{
					contractEvent(
						contractID,
						xdr.ScVec{
							counterScVal,
						},
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &datas[2]},
					),
					contractEvent(
						contractID,
						xdr.ScVec{
							counterScVal,
						},
						xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &datas[3]},
					),
				},
			),
		}
		assert.NoError(t, store.IngestEvents(ledgerCloseMetaWithEvents(5, now.Unix(), txMeta...)))

		id := events.Cursor{Ledger: 5, Tx: 1, Op: 0, Event: 0}.String()
		handler := eventsRPCHandler{
			scanner:      store,
			maxLimit:     10000,
			defaultLimit: 100,
		}
		results, err := handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Pagination: &PaginationOptions{
				Cursor: id,
				Limit:  2,
			},
		})
		assert.NoError(t, err)

		var expected []EventInfo
		expectedIDs := []string{
			events.Cursor{Ledger: 5, Tx: 1, Op: 0, Event: 1}.String(),
			events.Cursor{Ledger: 5, Tx: 1, Op: 1, Event: 0}.String(),
		}
		symbols := datas[1:3]
		for i, id := range expectedIDs {
			expectedXdr, err := xdr.MarshalBase64(xdr.ScVal{Type: xdr.ScValTypeScvSymbol, Sym: &symbols[i]})
			assert.NoError(t, err)
			expected = append(expected, EventInfo{
				EventType:      EventTypeContract,
				Ledger:         5,
				LedgerClosedAt: now.Format(time.RFC3339),
				ContractID:     contractID.HexString(),
				ID:             id,
				PagingToken:    id,
				Topic:          []string{counterXdr},
				Value:          EventInfoValue{XDR: expectedXdr},
			})
		}
		assert.Equal(t, expected, results)

		results, err = handler.getEvents(GetEventsRequest{
			StartLedger: 1,
			Pagination: &PaginationOptions{
				Cursor: events.Cursor{Ledger: 5, Tx: 1, Op: 1, Event: 1}.String(),
				Limit:  2,
			},
		})
		assert.NoError(t, err)
		assert.Empty(t, results)
	})
}

func ledgerCloseMetaWithEvents(sequence uint32, closeTimestamp int64, txMeta ...xdr.TransactionMeta) xdr.LedgerCloseMeta {
	var txProcessing []xdr.TransactionResultMetaV2
	var phases []xdr.TransactionPhase

	for _, item := range txMeta {
		var operations []xdr.Operation
		for range item.MustV3().Events {
			operations = append(operations,
				xdr.Operation{
					Body: xdr.OperationBody{
						Type: xdr.OperationTypeInvokeHostFunction,
						InvokeHostFunctionOp: &xdr.InvokeHostFunctionOp{
							Function: xdr.HostFunction{
								Type:       xdr.HostFunctionTypeHostFunctionTypeInvokeContract,
								InvokeArgs: &xdr.ScVec{},
							},
						},
					},
				})
		}
		envelope := xdr.TransactionEnvelope{
			Type: xdr.EnvelopeTypeEnvelopeTypeTx,
			V1: &xdr.TransactionV1Envelope{
				Tx: xdr.Transaction{
					SourceAccount: xdr.MustMuxedAddress(keypair.MustRandom().Address()),
					Operations:    operations,
				},
			},
		}
		txHash, err := network.HashTransactionInEnvelope(envelope, "unit-tests")
		if err != nil {
			panic(err)
		}

		txProcessing = append(txProcessing, xdr.TransactionResultMetaV2{
			TxApplyProcessing: item,
			Result: xdr.TransactionResultPairV2{
				TransactionHash: txHash,
			},
		})
		components := []xdr.TxSetComponent{
			{
				Type: xdr.TxSetComponentTypeTxsetCompTxsMaybeDiscountedFee,
				TxsMaybeDiscountedFee: &xdr.TxSetComponentTxsMaybeDiscountedFee{
					Txs: []xdr.TransactionEnvelope{
						envelope,
					},
				},
			},
		}
		phases = append(phases, xdr.TransactionPhase{
			V:            0,
			V0Components: &components,
		})
	}

	return xdr.LedgerCloseMeta{
		V: 2,
		V2: &xdr.LedgerCloseMetaV2{
			LedgerHeader: xdr.LedgerHeaderHistoryEntry{
				Hash: xdr.Hash{},
				Header: xdr.LedgerHeader{
					ScpValue: xdr.StellarValue{
						CloseTime: xdr.TimePoint(closeTimestamp),
					},
					LedgerSeq: xdr.Uint32(sequence),
				},
			},
			TxSet: xdr.GeneralizedTransactionSet{
				V: 1,
				V1TxSet: &xdr.TransactionSetV1{
					PreviousLedgerHash: xdr.Hash{},
					Phases:             phases,
				},
			},
			TxProcessing: txProcessing,
		},
	}
}
func transactionMetaWithEvents(events ...[]xdr.ContractEvent) xdr.TransactionMeta {
	var operationEvents []xdr.OperationEvents
	for _, e := range events {
		operationEvents = append(operationEvents, xdr.OperationEvents{
			Events: e,
		})
	}
	return xdr.TransactionMeta{
		V:          3,
		Operations: &[]xdr.OperationMeta{},
		V3: &xdr.TransactionMetaV3{
			TxResult: xdr.TransactionResult{
				Result: xdr.TransactionResultResult{
					InnerResultPair: &xdr.InnerTransactionResultPair{},
					Results:         &[]xdr.OperationResult{},
				},
			},
			Events: operationEvents,
		},
	}
}

func contractEvent(contractID xdr.Hash, topic []xdr.ScVal, body xdr.ScVal) xdr.ContractEvent {
	return xdr.ContractEvent{
		ContractId: &contractID,
		Type:       xdr.ContractEventTypeContract,
		Body: xdr.ContractEventBody{
			V: 0,
			V0: &xdr.ContractEventV0{
				Topics: xdr.ScVec(topic),
				Data:   body,
			},
		},
	}
}

func systemEvent(contractID xdr.Hash, topic []xdr.ScVal, body xdr.ScVal) xdr.ContractEvent {
	return xdr.ContractEvent{
		ContractId: &contractID,
		Type:       xdr.ContractEventTypeSystem,
		Body: xdr.ContractEventBody{
			V: 0,
			V0: &xdr.ContractEventV0{
				Topics: xdr.ScVec(topic),
				Data:   body,
			},
		},
	}
}
