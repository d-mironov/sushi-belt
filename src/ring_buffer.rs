use crate::error::Full;

#[cfg(test)]
mod tests;

/// A first-in, first-out queue that holds up to `N` elements without
/// allocating.
///
/// Elements are added at the back with [`push`](Self::push) and removed from
/// the front with [`pop`](Self::pop). When the buffer is full, new elements are
/// rejected rather than overwriting old ones.
///
/// # Examples
///
/// ```
/// use sushi_belt::RingBuffer;
///
/// let mut ring = RingBuffer::<u32, 4>::new();
///
/// ring.push(1).unwrap();
/// ring.push(2).unwrap();
///
/// assert_eq!(ring.peek(), Some(&1));
/// assert_eq!(ring.pop(), Some(1));
/// assert_eq!(ring.len(), 1);
/// ```
///
/// `new` is a `const fn`, so a buffer can be placed in a `static`:
///
/// ```
/// use sushi_belt::RingBuffer;
/// use std::sync::Mutex;
///
/// static RING: Mutex<RingBuffer<u32, 4>> = Mutex::new(RingBuffer::new());
///
/// RING.lock().unwrap().push(1).unwrap();
/// assert_eq!(RING.lock().unwrap().pop(), Some(1));
/// ```
///
/// # Debug output
///
/// The [`Debug`] output shows the internal slots in storage order, including
/// empty ones, rather than the elements in queue order.
///
/// # Zero capacity
///
/// `RingBuffer<T, 0>` is valid. It's always both empty and full, so every push
/// is rejected and every pop returns [`None`].
#[derive(Debug, Clone)]
pub struct RingBuffer<T, const N: usize> {
    buf: [Option<T>; N],
    write: usize,
    read: usize,
    size: usize,
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    /// Creates an empty buffer. This is the same as [`new`](Self::new).
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Creates an empty buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let ring = RingBuffer::<u32, 4>::new();
    ///
    /// assert!(ring.is_empty());
    /// assert_eq!(ring.capacity(), 4);
    /// ```
    pub const fn new() -> Self {
        Self {
            buf: [const { None }; N],
            write: 0,
            read: 0,
            size: 0,
        }
    }

    /// Creates a buffer holding clones of `items`, with the first item at the
    /// front of the queue.
    ///
    /// Returns [`None`] if there are more than `N` items.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::from_slice(&[1, 2]).unwrap();
    /// assert_eq!(ring.pop(), Some(1));
    ///
    /// assert!(RingBuffer::<u32, 1>::from_slice(&[1, 2]).is_none());
    /// ```
    pub fn from_slice(items: &[T]) -> Option<Self>
    where
        T: Clone,
    {
        let len = items.len();
        if len > N {
            return None;
        }

        Some(Self {
            buf: core::array::from_fn(|i| items.get(i).cloned()),
            read: 0,
            write: if len == N { 0 } else { len },
            size: len,
        })
    }

    /// Returns the number of elements in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::new();
    /// ring.push(1).unwrap();
    ///
    /// assert_eq!(ring.len(), 1);
    /// ```
    pub const fn len(&self) -> usize {
        self.size
    }

    /// Returns the maximum number of elements the buffer can hold, which is
    /// always `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let ring = RingBuffer::<u32, 4>::new();
    ///
    /// assert_eq!(ring.capacity(), 4);
    /// ```
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns how many more elements fit in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::new();
    /// ring.push(1).unwrap();
    ///
    /// assert_eq!(ring.free(), 3);
    /// ```
    pub const fn free(&self) -> usize {
        N - self.size
    }

    /// Returns `true` if the buffer holds no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::new();
    /// assert!(ring.is_empty());
    ///
    /// ring.push(1).unwrap();
    /// assert!(!ring.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns `true` if the buffer has no free space.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 1>::new();
    /// assert!(!ring.is_full());
    ///
    /// ring.push(1).unwrap();
    /// assert!(ring.is_full());
    /// ```
    pub const fn is_full(&self) -> bool {
        self.size == N
    }

    /// Returns a reference to the oldest element without removing it, or
    /// [`None`] if the buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::new();
    /// assert_eq!(ring.peek(), None);
    ///
    /// ring.push(1).unwrap();
    /// assert_eq!(ring.peek(), Some(&1));
    /// assert_eq!(ring.len(), 1);
    /// ```
    pub const fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.buf[self.read].as_ref()
    }

    /// Removes and returns the oldest element, or [`None`] if the buffer is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::new();
    /// ring.push(1).unwrap();
    /// ring.push(2).unwrap();
    ///
    /// assert_eq!(ring.pop(), Some(1));
    /// assert_eq!(ring.pop(), Some(2));
    /// assert_eq!(ring.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let retval = self.buf[self.read].take();
        self.read = (self.read + 1) % N;
        self.size -= 1;
        retval
    }

    /// Adds an element to the back of the queue.
    ///
    /// # Errors
    ///
    /// Returns the element inside [`Full`] if the buffer is full. The buffer
    /// isn't changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::{Full, RingBuffer};
    ///
    /// let mut ring = RingBuffer::<u32, 1>::new();
    ///
    /// assert_eq!(ring.push(1), Ok(()));
    /// assert_eq!(ring.push(2), Err(Full(2)));
    /// ```
    pub fn push(&mut self, val: T) -> Result<(), Full<T>> {
        if self.is_full() {
            return Err(Full(val));
        }
        self.buf[self.write] = Some(val);
        self.write = (self.write + 1) % N;
        self.size += 1;
        Ok(())
    }

    /// Adds clones of all `items` to the back of the queue, in order.
    ///
    /// This takes O(`items.len()`) time, because elements are cloned one at a
    /// time. If a call to `clone` panics, the items added before it stay in the
    /// buffer.
    ///
    /// # Errors
    ///
    /// If `items` doesn't fit in the free space, returns it inside [`Full`] and
    /// adds nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::{Full, RingBuffer};
    ///
    /// let mut ring = RingBuffer::<u32, 3>::new();
    /// let too_many = [3, 4];
    ///
    /// assert_eq!(ring.push_slice(&[1, 2]), Ok(()));
    /// assert_eq!(ring.push_slice(&too_many), Err(Full(&too_many[..])));
    /// assert_eq!(ring.len(), 2);
    /// ```
    pub fn push_slice<'a>(&mut self, items: &'a [T]) -> Result<(), Full<&'a [T]>>
    where
        T: Clone,
    {
        if items.len() > self.free() {
            return Err(Full(items));
        }
        for item in items {
            self.buf[self.write] = Some(item.clone());
            self.write = (self.write + 1) % N;
            self.size += 1;
        }
        Ok(())
    }

    /// Moves the oldest elements into `out`, in order, and returns how many
    /// were moved.
    ///
    /// The result is the smaller of `out.len()` and [`len`](Self::len), and is
    /// 0 if the buffer is empty. Any remaining elements stay in the buffer, and
    /// the rest of `out` isn't changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sushi_belt::RingBuffer;
    ///
    /// let mut ring = RingBuffer::<u32, 4>::from_slice(&[1, 2, 3]).unwrap();
    /// let mut out = [0; 2];
    ///
    /// assert_eq!(ring.pop_slice(&mut out), 2);
    /// assert_eq!(out, [1, 2]);
    ///
    /// assert_eq!(ring.pop_slice(&mut out), 1);
    /// assert_eq!(out[0], 3);
    ///
    /// assert_eq!(ring.pop_slice(&mut out), 0);
    /// ```
    pub fn pop_slice(&mut self, out: &mut [T]) -> usize {
        let mut count = 0;
        for slot in out.iter_mut() {
            let Some(val) = self.pop() else { break };
            *slot = val;
            count += 1;
        }
        count
    }
}
