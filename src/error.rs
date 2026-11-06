use core::{error::Error, fmt};

/// The error returned when a [`RingBuffer`](crate::RingBuffer) doesn't have
/// enough free space.
///
/// It holds whatever could not be added, so nothing is lost and the caller can
/// retry or discard it: an element for
/// [`push`](crate::RingBuffer::push), or the whole slice for
/// [`push_slice`](crate::RingBuffer::push_slice).
///
/// The [`Debug`] output doesn't include the value, so `T` doesn't need to
/// implement [`Debug`] for `unwrap` to work.
///
/// # Examples
///
/// Recover the rejected value by matching on the error:
///
/// ```
/// use sushi_belt::{Full, RingBuffer};
///
/// let mut ring = RingBuffer::<u32, 1>::new();
/// ring.push(1).unwrap();
///
/// if let Err(Full(rejected)) = ring.push(2) {
///     assert_eq!(rejected, 2);
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Full<T>(
    /// The value that could not be added.
    pub T,
);

impl<T> Full<T> {
    /// Returns the value that could not be added.
    ///
    /// This is useful with [`Result::map_err`] when you only want the value:
    ///
    /// ```
    /// use sushi_belt::{Full, RingBuffer};
    ///
    /// let mut ring = RingBuffer::<u32, 0>::new();
    ///
    /// assert_eq!(ring.push(1).map_err(Full::into_inner), Err(1));
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Full<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Full(..)")
    }
}

impl<T> fmt::Display for Full<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ring buffer is full")
    }
}

impl<T> Error for Full<T> {}

#[cfg(feature = "defmt")]
impl<T> defmt::Format for Full<T> {
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(f, "Full(..)")
    }
}
