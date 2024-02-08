package methods

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/creachadair/jrpc2"
	"github.com/creachadair/jrpc2/handler"

	"github.com/stellar/go/strkey"
	"github.com/stellar/go/support/errors"
	"github.com/stellar/go/xdr"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/events"
)

type eventTypeSet map[string]interface{}

func (e eventTypeSet) valid() error {
	for key := range e {
		switch key {
		case EventTypeSystem, EventTypeContract, EventTypeDiagnostic:
			// ok
		default:
			return errors.New("if set, type must be either 'system', 'contract' or 'diagnostic'")
		}
	}
	return nil
}

func (e *eventTypeSet) UnmarshalJSON(data []byte) error {
	if len(data) == 0 {
		*e = map[string]interface{}{}
		return nil
	}
	var joined string
	if err := json.Unmarshal(data, &joined); err != nil {
		return err
	}
	*e = map[string]interface{}{}
	if len(joined) == 0 {
		return nil
	}
	for _, key := range strings.Split(joined, ",") {
		(*e)[key] = nil
	}
	return nil
}

func (e eventTypeSet) MarshalJSON() ([]byte, error) {
	var keys []string
	for key := range e {
		keys = append(keys, key)
	}
	return json.Marshal(strings.Join(keys, ","))
}

func (e eventTypeSet) matches(event xdr.ContractEvent) bool {
	if len(e) == 0 {
		return true
	}
	_, ok := e[eventTypeFromXDR[event.Type]]
	return ok
}

type EventInfo struct {
	EventType                string   `json:"type"`
	Ledger                   int32    `json:"ledger"`
	LedgerClosedAt           string   `json:"ledgerClosedAt"`
	ContractID               string   `json:"contractId"`
	ID                       string   `json:"id"`
	PagingToken              string   `json:"pagingToken"`
	Topic                    []string `json:"topic"`
	Value                    string   `json:"value"`
	InSuccessfulContractCall bool     `json:"inSuccessfulContractCall"`
	TransactionHash          string   `json:"txHash"`
}

type GetEventsRequest struct {
	StartLedger uint32             `json:"startLedger,omitempty"`
	Filters     []EventFilter      `json:"filters"`
	Pagination  *PaginationOptions `json:"pagination,omitempty"`
}

func (g *GetEventsRequest) Valid(maxLimit uint) error {
	// Validate start
	// Validate the paging limit (if it exists)
	if g.Pagination != nil && g.Pagination.Cursor != nil {
		if g.StartLedger != 0 {
			return errors.New("startLedger and cursor cannot both be set")
		}
	} else if g.StartLedger <= 0 {
		return errors.New("startLedger must be positive")
	}
	if g.Pagination != nil && g.Pagination.Limit > maxLimit {
		return fmt.Errorf("limit must not exceed %d", maxLimit)
	}

	// Validate filters
	if len(g.Filters) > 5 {
		return errors.New("maximum 5 filters per request")
	}
	for i, filter := range g.Filters {
		if err := filter.Valid(); err != nil {
			return errors.Wrapf(err, "filter %d invalid", i+1)
		}
	}

	return nil
}

func (g *GetEventsRequest) Matches(event xdr.DiagnosticEvent) bool {
	if len(g.Filters) == 0 {
		return true
	}
	for _, filter := range g.Filters {
		if filter.Matches(event) {
			return true
		}
	}
	return false
}

const EventTypeSystem = "system"
const EventTypeContract = "contract"
const EventTypeDiagnostic = "diagnostic"

var eventTypeFromXDR = map[xdr.ContractEventType]string{
	xdr.ContractEventTypeSystem:     EventTypeSystem,
	xdr.ContractEventTypeContract:   EventTypeContract,
	xdr.ContractEventTypeDiagnostic: EventTypeDiagnostic,
}

type EventFilter struct {
	EventType   eventTypeSet  `json:"type,omitempty"`
	ContractIDs []string      `json:"contractIds,omitempty"`
	Topics      []TopicFilter `json:"topics,omitempty"`
}

func (e *EventFilter) Valid() error {
	if err := e.EventType.valid(); err != nil {
		return errors.Wrap(err, "filter type invalid")
	}
	if len(e.ContractIDs) > 5 {
		return errors.New("maximum 5 contract IDs per filter")
	}
	if len(e.Topics) > 5 {
		return errors.New("maximum 5 topics per filter")
	}
	for i, id := range e.ContractIDs {
		_, err := strkey.Decode(strkey.VersionByteContract, id)
		if err != nil {
			return fmt.Errorf("contract ID %d invalid", i+1)
		}
	}
	for i, topic := range e.Topics {
		if err := topic.Valid(); err != nil {
			return errors.Wrapf(err, "topic %d invalid", i+1)
		}
	}
	return nil
}

func (e *EventFilter) Matches(event xdr.DiagnosticEvent) bool {
	return e.EventType.matches(event.Event) && e.matchesContractIDs(event.Event) && e.matchesTopics(event.Event)
}

func (e *EventFilter) matchesContractIDs(event xdr.ContractEvent) bool {
	if len(e.ContractIDs) == 0 {
		return true
	}
	if event.ContractId == nil {
		return false
	}
	needle := strkey.MustEncode(strkey.VersionByteContract, (*event.ContractId)[:])
	for _, id := range e.ContractIDs {
		if id == needle {
			return true
		}
	}
	return false
}

func (e *EventFilter) matchesTopics(event xdr.ContractEvent) bool {
	if len(e.Topics) == 0 {
		return true
	}
	v0, ok := event.Body.GetV0()
	if !ok {
		return false
	}
	for _, topicFilter := range e.Topics {
		if topicFilter.Matches(v0.Topics) {
			return true
		}
	}
	return false
}

type TopicFilter []SegmentFilter

const minTopicCount = 1
const maxTopicCount = 4

func (t *TopicFilter) Valid() error {
	if len(*t) < minTopicCount {
		return errors.New("topic must have at least one segment")
	}
	if len(*t) > maxTopicCount {
		return errors.New("topic cannot have more than 4 segments")
	}
	for i, segment := range *t {
		if err := segment.Valid(); err != nil {
			return errors.Wrapf(err, "segment %d invalid", i+1)
		}
	}
	return nil
}

// An event matches a topic filter iff:
//   - the event has EXACTLY as many topic segments as the filter AND
//   - each segment either: matches exactly OR is a wildcard.
func (t TopicFilter) Matches(event []xdr.ScVal) bool {
	if len(event) != len(t) {
		return false
	}

	for i, segmentFilter := range t {
		if !segmentFilter.Matches(event[i]) {
			return false
		}
	}

	return true
}

type SegmentFilter struct {
	wildcard *string
	scval    *xdr.ScVal
}

func (s *SegmentFilter) Matches(segment xdr.ScVal) bool {
	if s.wildcard != nil && *s.wildcard == "*" {
		return true
	} else if s.scval != nil {
		if !s.scval.Equals(segment) {
			return false
		}
	} else {
		panic("invalid segmentFilter")
	}

	return true
}

func (s *SegmentFilter) Valid() error {
	if s.wildcard != nil && s.scval != nil {
		return errors.New("cannot set both wildcard and scval")
	}
	if s.wildcard == nil && s.scval == nil {
		return errors.New("must set either wildcard or scval")
	}
	if s.wildcard != nil && *s.wildcard != "*" {
		return errors.New("wildcard must be '*'")
	}
	return nil
}

func (s *SegmentFilter) UnmarshalJSON(p []byte) error {
	s.wildcard = nil
	s.scval = nil

	var tmp string
	if err := json.Unmarshal(p, &tmp); err != nil {
		return err
	}
	if tmp == "*" {
		s.wildcard = &tmp
	} else {
		var out xdr.ScVal
		if err := xdr.SafeUnmarshalBase64(tmp, &out); err != nil {
			return err
		}
		s.scval = &out
	}
	return nil
}

type PaginationOptions struct {
	Cursor *events.Cursor `json:"cursor,omitempty"`
	Limit  uint           `json:"limit,omitempty"`
}

type GetEventsResponse struct {
	Events       []EventInfo `json:"events"`
	LatestLedger uint32      `json:"latestLedger"`
}

type eventScanner interface {
	Scan(eventRange events.Range, f events.ScanFunction) (uint32, error)
}

type eventsRPCHandler struct {
	scanner      eventScanner
	maxLimit     uint
	defaultLimit uint
}

func (h eventsRPCHandler) getEvents(request GetEventsRequest) (GetEventsResponse, error) {
	if err := request.Valid(h.maxLimit); err != nil {
		return GetEventsResponse{}, &jrpc2.Error{
			Code:    jrpc2.InvalidParams,
			Message: err.Error(),
		}
	}

	start := events.Cursor{Ledger: uint32(request.StartLedger)}
	limit := h.defaultLimit
	if request.Pagination != nil {
		if request.Pagination.Cursor != nil {
			start = *request.Pagination.Cursor
			// increment event index because, when paginating,
			// we start with the item right after the cursor
			start.Event++
		}
		if request.Pagination.Limit > 0 {
			limit = request.Pagination.Limit
		}
	}

	type entry struct {
		cursor               events.Cursor
		ledgerCloseTimestamp int64
		event                xdr.DiagnosticEvent
		txHash               *xdr.Hash
	}
	var found []entry
	latestLedger, err := h.scanner.Scan(
		events.Range{
			Start:      start,
			ClampStart: false,
			End:        events.MaxCursor,
			ClampEnd:   true,
		},
		func(event xdr.DiagnosticEvent, cursor events.Cursor, ledgerCloseTimestamp int64, txHash *xdr.Hash) bool {
			if request.Matches(event) {
				found = append(found, entry{cursor, ledgerCloseTimestamp, event, txHash})
			}
			return uint(len(found)) < limit
		},
	)
	if err != nil {
		return GetEventsResponse{}, &jrpc2.Error{
			Code:    jrpc2.InvalidRequest,
			Message: err.Error(),
		}
	}

	results := []EventInfo{}
	for _, entry := range found {
		info, err := eventInfoForEvent(
			entry.event,
			entry.cursor,
			time.Unix(entry.ledgerCloseTimestamp, 0).UTC().Format(time.RFC3339),
			entry.txHash.HexString(),
		)
		if err != nil {
			return GetEventsResponse{}, errors.Wrap(err, "could not parse event")
		}
		results = append(results, info)
	}
	return GetEventsResponse{
		LatestLedger: latestLedger,
		Events:       results,
	}, nil
}

func eventInfoForEvent(event xdr.DiagnosticEvent, cursor events.Cursor, ledgerClosedAt string, txHash string) (EventInfo, error) {
	v0, ok := event.Event.Body.GetV0()
	if !ok {
		return EventInfo{}, errors.New("unknown event version")
	}

	eventType, ok := eventTypeFromXDR[event.Event.Type]
	if !ok {
		return EventInfo{}, fmt.Errorf("unknown XDR ContractEventType type: %d", event.Event.Type)
	}

	// base64-xdr encode the topic
	topic := make([]string, 0, 4)
	for _, segment := range v0.Topics {
		seg, err := xdr.MarshalBase64(segment)
		if err != nil {
			return EventInfo{}, err
		}
		topic = append(topic, seg)
	}

	// base64-xdr encode the data
	data, err := xdr.MarshalBase64(v0.Data)
	if err != nil {
		return EventInfo{}, err
	}

	info := EventInfo{
		EventType:                eventType,
		Ledger:                   int32(cursor.Ledger),
		LedgerClosedAt:           ledgerClosedAt,
		ID:                       cursor.String(),
		PagingToken:              cursor.String(),
		Topic:                    topic,
		Value:                    data,
		InSuccessfulContractCall: event.InSuccessfulContractCall,
		TransactionHash:          txHash,
	}
	if event.Event.ContractId != nil {
		info.ContractID = strkey.MustEncode(strkey.VersionByteContract, (*event.Event.ContractId)[:])
	}
	return info, nil
}

// NewGetEventsHandler returns a json rpc handler to fetch and filter events
func NewGetEventsHandler(eventsStore *events.MemoryStore, maxLimit, defaultLimit uint) jrpc2.Handler {
	eventsHandler := eventsRPCHandler{
		scanner:      eventsStore,
		maxLimit:     maxLimit,
		defaultLimit: defaultLimit,
	}
	return handler.New(func(ctx context.Context, request GetEventsRequest) (GetEventsResponse, error) {
		return eventsHandler.getEvents(request)
	})
}
