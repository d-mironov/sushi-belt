# sushi-belt 🍣

**A fixed-capacity ring buffer for embedded Rust, written entirely in safe code.**

Think of a conveyor belt at a sushi bar. It has a fixed number of spots, plates come off in the order they went on, and when every spot is taken, the next plate has to wait.

`sushi-belt` is a first-in, first-out queue that lives wherever you put it: on the stack or in a `static`. It never allocates, needs only `core`, and forbids `unsafe` at compile time.

- **No allocator:** capacity is part of the type, so all storage is reserved up front.
- **No `unsafe`:** enforced with `#![forbid(unsafe_code)]`.
- **Usable in a `static`:** `RingBuffer::new` is a `const fn`.
- **Nothing is lost on failure:** a rejected value is handed back to you inside the error.
- **Any element type:** `T` doesn't need to be `Copy`, `Clone` or `Default` for `push` and `pop`.
- **Optional `defmt` support:** log errors efficiently on embedded targets.

## Quick start

```toml
[dependencies]
sushi-belt = "0.1"
```

```rust
use sushi_belt::{Full, RingBuffer};

let mut ring = RingBuffer::<u32, 2>::new();

ring.push(1).unwrap();
ring.push(2).unwrap();

// The buffer is full, so the rejected value comes back in the error.
assert_eq!(ring.push(3), Err(Full(3)));

// Elements leave in the order they arrived.
assert_eq!(ring.pop(), Some(1));
assert_eq!(ring.pop(), Some(2));
assert_eq!(ring.pop(), None);
```

## Sharing with an interrupt

Every method that changes the buffer takes `&mut self`, so sharing one between an interrupt and your main code needs a lock. This example uses the [`critical-section`](https://crates.io/crates/critical-section) crate:

```rust
use sushi_belt::RingBuffer;
use core::cell::RefCell;
use critical_section::Mutex;

static SAMPLES: Mutex<RefCell<RingBuffer<u16, 64>>> =
    Mutex::new(RefCell::new(RingBuffer::new()));

fn on_adc_interrupt(sample: u16) {
    critical_section::with(|cs| {
        // Drop the sample if the consumer has fallen behind.
        let _ = SAMPLES.borrow_ref_mut(cs).push(sample);
    });
}

fn next_sample() -> Option<u16> {
    critical_section::with(|cs| SAMPLES.borrow_ref_mut(cs).pop())
}

on_adc_interrupt(42);
assert_eq!(next_sample(), Some(42));
assert_eq!(next_sample(), None);
```

## Working with slices

`push_slice` is all-or-nothing, while `pop_slice` takes as many elements as are available:

```rust
use sushi_belt::RingBuffer;

let mut ring = RingBuffer::<u8, 4>::new();

ring.push_slice(b"abc").unwrap();

// Only one slot is free, so nothing from this slice is added.
assert!(ring.push_slice(b"de").is_err());
assert_eq!(ring.len(), 3);

let mut out = [0; 8];
let count = ring.pop_slice(&mut out);
assert_eq!(&out[..count], b"abc");
```

These methods copy one element at a time, so they aren't a `memcpy`.

## API at a glance

| Method | Description | Time |
|---|---|---|
| `new` | Creates an empty buffer (`const`) | O(N) |
| `from_slice` | Creates a buffer from clones of a slice | O(N) |
| `push` | Adds an element, or returns it in `Full` | O(1) |
| `pop` | Removes the oldest element | O(1) |
| `peek` | Borrows the oldest element | O(1) |
| `push_slice` | Adds a whole slice, or nothing | O(n) |
| `pop_slice` | Moves up to `out.len()` elements out | O(n) |
| `len`, `capacity`, `free` | Report how full the buffer is | O(1) |
| `is_empty`, `is_full` | Report the buffer's state | O(1) |

## Memory use

Each slot is an `Option<T>`, which is what lets this crate avoid `unsafe`. The cost is `N * size_of::<Option<T>>()` plus three `usize` counters:

| Type (on a 32-bit target) | Slot size | Total size |
|---|---:|---:|
| `RingBuffer<u8, 64>` | 2 bytes | 140 bytes |
| `RingBuffer<u32, 64>` | 8 bytes | 524 bytes |
| `RingBuffer<&'static str, 64>` | 8 bytes | 524 bytes |

Types with a spare bit pattern, like references, `NonZeroU32` and `Box`, have an `Option` no bigger than the type itself.

## Is this the right crate for you?

Pick `sushi-belt` if you want a small, simple queue that's free of `unsafe` and you can afford the extra bytes per slot. Otherwise, another crate may suit you better:

| You need | Consider |
|---|---|
| The smallest memory footprint, double-ended access or iteration | [`heapless::Deque`](https://docs.rs/heapless/latest/heapless/deque/type.Deque.html) |
| A lock-free queue between one producer and one consumer | [`heapless::spsc::Queue`](https://docs.rs/heapless/latest/heapless/spsc/index.html) |
| Old data overwritten when the buffer fills up | [`heapless::HistoryBuf`](https://docs.rs/heapless/latest/heapless/history_buf/type.HistoryBuf.html) or [`circular-buffer`](https://crates.io/crates/circular-buffer) |

## Feature flags

| Feature | Description |
|---|---|
| `defmt` | Implements `defmt::Format` for the `Full` error |

## License

Licensed under the MIT License. See the `LICENSE` file for details.
