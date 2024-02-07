package ledgerbucketwindow

import (
	"fmt"
)

// LedgerBucketWindow is a sequence of buckets associated to a ledger window.
type LedgerBucketWindow[T any] struct {
	// buckets is a circular buffer where each cell represents
	// the content stored for a specific ledger.
	buckets []LedgerBucket[T]
	// start is the index of the head in the circular buffer.
	start uint32
}

// LedgerBucket holds the content associated to a ledger
type LedgerBucket[T any] struct {
	LedgerSeq            uint32
	LedgerCloseTimestamp int64
	BucketContent        T
}

// DefaultEventLedgerRetentionWindow represents the max number of ledgers we would like to keep
// an incoming event in memory. The value was calculated to align with (roughly) 24 hours window.
const DefaultEventLedgerRetentionWindow = 17280

// DefaultLedgerRetentionWindow represents default number of ledgers we would like to keep for events and transactions.
// The value was calculated to align with (roughly) 24 hours window.
const DefaultLedgerRetentionWindow = 17280

// NewLedgerBucketWindow creates a new LedgerBucketWindow
func NewLedgerBucketWindow[T any](retentionWindow uint32) *LedgerBucketWindow[T] {
	if retentionWindow == 0 {
		retentionWindow = DefaultLedgerRetentionWindow
	}
	return &LedgerBucketWindow[T]{
		buckets: make([]LedgerBucket[T], 0, retentionWindow),
	}
}

// Append adds a new bucket to the window. If the window is full a bucket will be evicted and returned.
func (w *LedgerBucketWindow[T]) Append(bucket LedgerBucket[T]) (*LedgerBucket[T], error) {
	length := w.Len()
	if length > 0 {
		expectedLedgerSequence := w.buckets[w.start].LedgerSeq + length
		if expectedLedgerSequence != bucket.LedgerSeq {
			return &LedgerBucket[T]{}, fmt.Errorf("error appending ledgers: ledgers not contiguous: expected ledger sequence %v but received %v", expectedLedgerSequence, bucket.LedgerSeq)
		}
	}

	var evicted *LedgerBucket[T]
	if length < uint32(cap(w.buckets)) {
		// The buffer isn't full, just place the bucket at the end
		w.buckets = append(w.buckets, bucket)
	} else {
		// overwrite the first bucket and shift the circular buffer so that it
		// becomes the last bucket
		saved := w.buckets[w.start]
		evicted = &saved
		w.buckets[w.start] = bucket
		w.start = (w.start + 1) % length
	}

	return evicted, nil
}

// Len returns the length (number of buckets in the window)
func (w *LedgerBucketWindow[T]) Len() uint32 {
	return uint32(len(w.buckets))
}

// Get obtains a bucket from the window
func (w *LedgerBucketWindow[T]) Get(i uint32) *LedgerBucket[T] {
	length := w.Len()
	if i >= length {
		panic(fmt.Errorf("index out of range [%d] with length %d", i, length))
	}
	index := (w.start + i) % length
	return &w.buckets[index]
}
