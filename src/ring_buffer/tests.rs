use super::*;

#[test]
fn default_construction() {
    // Arrange
    const N: usize = 4;

    // Act
    // [`RingBuffer::default()`] just calls [`RingBuffer::new()`].
    let buf: RingBuffer<u32, N> = RingBuffer::new();

    // Assert
    assert_eq!(buf.buf, [None; N]);
}

#[test]
fn construct_from_slice() {
    // Arrange
    const N: usize = 4;
    let items: [u32; N] = [1, 2, 3, 4];
    let expected_buf: [Option<u32>; N] = items.map(Some);

    // Act
    let retval = RingBuffer::<u32, N>::from_slice(&items);
    assert!(retval.is_some());
    let rbuf = retval.unwrap();
    assert_eq!(rbuf.size, 4);
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.write, 0);
    assert_eq!(rbuf.read, 0);
}

#[test]
fn simple_push() {
    // Arrange
    const N: usize = 8;
    type PushResult = Result<(), Full<u32>>;
    let mut rbuf: RingBuffer<u32, N> = RingBuffer::new();
    let items: [u32; N] = [1, 2, 3, 4, 5, 6, 7, 8];

    let expected_buf: [Option<u32>; N] = items.map(Some);

    // Act
    let retvals: [PushResult; N] = items.map(|item| rbuf.push(item));

    // Assert
    assert!(retvals.iter().all(Result::is_ok));
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, N);
    assert_eq!(rbuf.write, 0);
    assert_eq!(rbuf.read, 0);
}

#[test]
fn push_full_buffer() {
    // Arrange
    const N: usize = 4;
    const VAL: u32 = 5;
    let items: [u32; N] = [4, 3, 2, 1];
    let mut rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");
    let expected_buf: [Option<u32>; N] = items.map(Some);

    // Act
    let retval = rbuf.push(VAL);

    // Assert
    assert_eq!(retval, Err(Full(VAL)));
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.write, 0);
    assert_eq!(rbuf.read, 0);
    assert_eq!(rbuf.size, N);
}

#[test]
fn peek_value() {
    // Arrange
    const N: usize = 4;
    let items: [u32; N] = [1, 2, 3, 4];
    let rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");

    // Act
    let retval = rbuf.peek();

    // Assert
    assert!(retval.is_some());
    let value = retval.unwrap();
    assert_eq!(value, &items[0]);
}

#[test]
fn pop_value() {
    // Arrange
    const N: usize = 4;
    let items: [u32; N] = [1, 2, 3, 4];
    let mut rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");
    let mut expected_buf: [Option<u32>; N] = items.map(Some);
    expected_buf[0] = None;

    // Act
    let retval = rbuf.pop();

    // Assert
    assert!(retval.is_some());
    let value = retval.unwrap();
    assert_eq!(value, items[0]);
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, N - 1);
    assert_eq!(rbuf.read, 1);
    assert_eq!(rbuf.write, 0);
}

#[test]
fn pop_push_after_full() {
    // Arrange
    const N: usize = 8;
    let items: [u32; N] = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");

    // Act
    let pop_retval = rbuf.pop();
    assert!(pop_retval.is_some());
    let pop_value = pop_retval.unwrap();
    assert_eq!(pop_value, items[0]);
    let push_retval = rbuf.push(9);
    assert!(push_retval.is_ok());
    assert_eq!(push_retval.unwrap(), ());
}

#[test]
fn drain_after_wraparound() {
    const N: usize = 8;
    let items: [u32; N] = core::array::from_fn(|i| i as u32 + 1);
    let mut rbuf = RingBuffer::<u32, N>::from_slice(&items).unwrap();

    assert_eq!(rbuf.pop(), Some(1));
    assert_eq!(rbuf.push(9), Ok(()));

    for expected in 2..=9 {
        assert_eq!(rbuf.pop(), Some(expected));
    }
    assert!(rbuf.is_empty());
    assert_eq!(rbuf.pop(), None);
}

#[test]
fn push_slice_wraps_and_rejects() {
    const N: usize = 4;
    let mut rbuf = RingBuffer::<u32, N>::new();

    assert!(rbuf.push_slice(&[1, 2, 3]).is_ok());
    assert_eq!(rbuf.pop(), Some(1));
    assert_eq!(rbuf.pop(), Some(2));
    assert!(rbuf.push_slice(&[4, 5, 6]).is_ok());
    assert!(rbuf.push_slice(&[7]).is_err());

    for expected in 3..=6 {
        assert_eq!(rbuf.pop(), Some(expected));
    }
    assert!(rbuf.is_empty());
}

#[test]
fn pop_slice_partial_and_wrapped() {
    const N: usize = 4;
    let mut rbuf = RingBuffer::<u32, N>::new();
    assert!(rbuf.push_slice(&[1, 2, 3, 4]).is_ok());

    let mut out = [0u32; 2];
    assert_eq!(rbuf.pop_slice(&mut out), 2);
    assert_eq!(out, [1, 2]);

    assert!(rbuf.push_slice(&[5, 6]).is_ok());

    let mut out = [0u32; 8];
    assert_eq!(rbuf.pop_slice(&mut out), 4);
    assert_eq!(out[..4], [3, 4, 5, 6]);
    assert!(rbuf.is_empty());
    assert_eq!(rbuf.pop_slice(&mut out), 0);
}

#[test]
fn default_matches_new() {
    // Arrange
    const N: usize = 4;

    // Act
    let rbuf: RingBuffer<u32, N> = RingBuffer::default();

    // Assert
    assert_eq!(rbuf.buf, [None; N]);
    assert_eq!(rbuf.size, 0);
    assert_eq!(rbuf.write, 0);
    assert_eq!(rbuf.read, 0);
}

#[test]
fn construct_from_partial_slice() {
    // Arrange
    const N: usize = 4;
    let items: [u32; 2] = [1, 2];
    let expected_buf: [Option<u32>; N] = [Some(1), Some(2), None, None];

    // Act
    let retval = RingBuffer::<u32, N>::from_slice(&items);

    // Assert
    assert!(retval.is_some());
    let rbuf = retval.unwrap();
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, items.len());
    assert_eq!(rbuf.write, items.len());
    assert_eq!(rbuf.read, 0);
}

#[test]
fn construct_from_oversized_slice() {
    // Arrange
    const N: usize = 4;
    let items: [u32; N + 1] = [1, 2, 3, 4, 5];

    // Act
    let retval = RingBuffer::<u32, N>::from_slice(&items);

    // Assert
    assert!(retval.is_none());
}

#[test]
fn peek_empty_buffer() {
    // Arrange
    const N: usize = 4;
    let rbuf: RingBuffer<u32, N> = RingBuffer::new();

    // Act
    let retval = rbuf.peek();

    // Assert
    assert_eq!(retval, None);
}

#[test]
fn peek_does_not_consume() {
    // Arrange
    const N: usize = 4;
    let items: [u32; N] = [1, 2, 3, 4];
    let rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");
    let expected_buf: [Option<u32>; N] = items.map(Some);

    // Act
    let first = rbuf.peek();
    let second = rbuf.peek();

    // Assert
    assert_eq!(first, Some(&items[0]));
    assert_eq!(second, Some(&items[0]));
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, N);
    assert_eq!(rbuf.read, 0);
}

#[test]
fn push_slice_oversized_leaves_buffer_unchanged() {
    // Arrange
    const N: usize = 4;
    let items: [u32; 3] = [1, 2, 3];
    let rejected: [u32; 2] = [4, 5];
    let mut rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");
    let expected_buf: [Option<u32>; N] = [Some(1), Some(2), Some(3), None];

    // Act
    let retval = rbuf.push_slice(&rejected);

    // Assert
    assert_eq!(retval, Err(Full(&rejected[..])));
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, items.len());
    assert_eq!(rbuf.write, items.len());
    assert_eq!(rbuf.read, 0);
}

#[test]
fn push_empty_slice_on_full_buffer() {
    // Arrange
    const N: usize = 2;
    let items: [u32; N] = [1, 2];
    let mut rbuf: RingBuffer<u32, N> =
        RingBuffer::from_slice(&items).expect("Failed to create RingBuffer!");
    let expected_buf: [Option<u32>; N] = items.map(Some);

    // Act
    let retval = rbuf.push_slice(&[]);

    // Assert
    assert_eq!(retval, Ok(()));
    assert_eq!(rbuf.buf, expected_buf);
    assert_eq!(rbuf.size, N);
    assert_eq!(rbuf.write, 0);
    assert_eq!(rbuf.read, 0);
}

#[test]
fn len_capacity_free_track_contents() {
    // Arrange
    const N: usize = 4;
    let mut rbuf: RingBuffer<u32, N> = RingBuffer::new();

    // Act & Assert: empty
    assert_eq!(rbuf.len(), 0);
    assert_eq!(rbuf.capacity(), N);
    assert_eq!(rbuf.free(), N);

    // Act & Assert: partly full
    assert!(rbuf.push_slice(&[1, 2, 3]).is_ok());
    assert_eq!(rbuf.len(), 3);
    assert_eq!(rbuf.capacity(), N);
    assert_eq!(rbuf.free(), 1);

    // Act & Assert: after a pop
    assert_eq!(rbuf.pop(), Some(1));
    assert_eq!(rbuf.len(), 2);
    assert_eq!(rbuf.capacity(), N);
    assert_eq!(rbuf.free(), 2);
}

#[test]
fn zero_capacity_construction() {
    // Arrange
    const N: usize = 0;
    let items: [u32; N] = [];

    // Act
    let from_new: RingBuffer<u32, N> = RingBuffer::new();
    let from_slice = RingBuffer::<u32, N>::from_slice(&items);

    // Assert
    assert!(from_new.is_empty());
    assert!(from_new.is_full());
    assert_eq!(from_new.capacity(), 0);
    assert_eq!(from_new.free(), 0);
    assert!(from_slice.is_some());
}

#[test]
fn zero_capacity_push_and_pop() {
    // Arrange
    const N: usize = 0;
    const VAL: u32 = 1;
    let mut rbuf: RingBuffer<u32, N> = RingBuffer::new();

    // Act
    let push_retval = rbuf.push(VAL);
    let peek_retval = rbuf.peek().copied();
    let pop_retval = rbuf.pop();

    // Assert
    assert_eq!(push_retval, Err(Full(VAL)));
    assert_eq!(peek_retval, None);
    assert_eq!(pop_retval, None);
}

#[test]
fn zero_capacity_slices() {
    // Arrange
    const N: usize = 0;
    let rejected: [u32; 1] = [1];
    let mut rbuf: RingBuffer<u32, N> = RingBuffer::new();
    let mut out = [0u32; 2];

    // Act
    let push_empty_retval = rbuf.push_slice(&[]);
    let push_retval = rbuf.push_slice(&rejected);
    let pop_count = rbuf.pop_slice(&mut out);

    // Assert
    assert_eq!(push_empty_retval, Ok(()));
    assert_eq!(push_retval, Err(Full(&rejected[..])));
    assert_eq!(pop_count, 0);
    assert_eq!(out, [0, 0]);
}
