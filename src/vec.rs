use core::{fmt, hash, mem::MaybeUninit/*, iter::FromIterator*/, ops, ptr, slice};

/// A fixed capacity [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html)
///
/// # Examples
///
/// ```
/// use heapless::Vec;
///
/// // A vector with a fixed capacity of 8 elements allocated on the stack
/// let mut vec = Vec::<_, 8>::new();
/// vec.push(1);
/// vec.push(2);
///
/// assert_eq!(vec.len(), 2);
/// assert_eq!(vec[0], 1);
///
/// assert_eq!(vec.pop(), Some(2));
/// assert_eq!(vec.len(), 1);
///
/// vec[0] = 7;
/// assert_eq!(vec[0], 7);
///
/// vec.extend([1, 2, 3].iter().cloned());
///
/// for x in &vec {
///     println!("{}", x);
/// }
/// assert_eq!(vec, [7, 1, 2, 3]);
/// ```
pub struct Vec<T, const N: usize> {
    pub(crate) buffer: MaybeUninit<[T; {N}]>,
    pub(crate) len: usize,
}

impl<T, const N: usize> Vec<T, {N}> {
    /// Constructs a new, empty vector with a fixed capacity of `N`
    ///
    /// # Examples
    ///
    /// ```
    /// use heapless::Vec;
    ///
    /// // allocate the vector on the stack
    /// let mut x: Vec<u8, 16> = Vec::new();
    ///
    /// // allocate the vector in a static variable
    /// static mut X: Vec<u8, 16> = Vec(heapless::i::Vec::new());
    /// ```
    pub const fn new() -> Self {
        Self {
            buffer: MaybeUninit::<[T; {N}]>::uninit(),
            len: 0,
        }
    }

    /// Returns the maximum number of elements the vector can hold
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Clears the vector, removing all values.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Clones and appends all elements in a slice to the `Vec`.
    ///
    /// Iterates over the slice `other`, clones each element, and then appends
    /// it to this `Vec`. The `other` vector is traversed in-order.
    ///
    /// # Examples
    ///
    /// ```
    /// use heapless::Vec;
    ///
    /// let mut vec = Vec::<u8, 8>::new();
    /// vec.push(1).unwrap();
    /// vec.extend_from_slice(&[2, 3, 4]).unwrap();
    /// assert_eq!(*vec, [1, 2, 3, 4]);
    /// ```
    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
    where
        T: Clone,
    {
        if self.len + other.len() > self.capacity() {
            // won't fit in the `Vec`; don't modify anything and return an error
            Err(())
        } else {
            for elem in other {
                unsafe {
                    self.push_unchecked(elem.clone());
                }
            }
            Ok(())
        }
    }

    pub(crate) fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for elem in iter {
            self.push(elem).ok().unwrap()
        }
    }

    /// Removes the last element from a vector and return it, or `None` if it's empty
    pub fn pop(&mut self) -> Option<T> {
        if self.len != 0 {
            Some(unsafe { self.pop_unchecked() })
        } else {
            None
        }
    }

    /// Appends an `item` to the back of the collection
    ///
    /// Returns back the `item` if the vector is full
    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.len < self.capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(item)
        }
    }

    pub(crate) unsafe fn push_unchecked(&mut self, item: T) {
        // NOTE(ptr::write) the memory slot that we are about to write to is uninitialized. We
        // use `ptr::write` to avoid running `T`'s destructor on the uninitialized memory
        (self.buffer.as_mut_ptr() as *mut T)
            .add(self.len)
            .write(item);

        self.len += 1;
    }

    pub(crate) unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.as_slice().is_empty());

        self.len -= 1;
        (self.buffer.as_ptr() as *const T).add(self.len).read()
    }

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            // drop any extra elements
            while len < self.len {
                // decrement len before the drop_in_place(), so a panic on Drop
                // doesn't re-drop the just-failed value.
                self.len -= 1;
                let len = self.len;
                ptr::drop_in_place(self.as_mut_slice().get_unchecked_mut(len));
            }
        }
    }

    /// Resizes the Vec in-place so that len is equal to new_len.
    ///
    /// If new_len is greater than len, the Vec is extended by the
    /// difference, with each additional slot filled with value. If
    /// new_len is less than len, the Vec is simply truncated.
    ///
    /// See also [`resize_default`](struct.Vec.html#method.resize_default).
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), ()>
    where
        T: Clone,
    {
        if new_len > self.capacity() {
            return Err(());
        }

        if new_len > self.len() {
            while self.len() < new_len {
                self.push(value.clone()).ok();
            }
        } else {
            self.truncate(new_len);
        }

        Ok(())
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `Default::default()`.
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// See also [`resize`](struct.Vec.html#method.resize).
    pub fn resize_default(&mut self, new_len: usize) -> Result<(), ()>
    where
        T: Clone + Default,
    {
        self.resize(new_len, T::default())
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use heapless::Vec;
    ///
    /// let mut v: Vec<_, 8> = Vec::new();
    /// v.push("foo").unwrap();
    /// v.push("bar").unwrap();
    /// v.push("baz").unwrap();
    /// v.push("qux").unwrap();
    ///
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(&*v, ["foo", "qux", "baz"]);
    ///
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(&*v, ["baz", "qux"]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len);
        unsafe { self.swap_remove_unchecked(index) }
    }

    pub(crate) unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        let length = self.len;
        debug_assert!(index < length);
        ptr::swap(
            self.as_mut_slice().get_unchecked_mut(index),
            self.as_mut_slice().get_unchecked_mut(length - 1),
        );
        self.pop_unchecked()
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &buffer[..self.len]
        unsafe { slice::from_raw_parts(self.buffer.as_ptr() as *const T, self.len) }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &mut buffer[..len]
        unsafe { slice::from_raw_parts_mut(self.buffer.as_mut_ptr() as *mut T, self.len) }
    }
/*
    pub(crate) fn clone(&self) -> Self
    where
        T: Clone,
    {
        let mut new = Self::new();
        new.extend_from_slice(self.as_slice()).unwrap();
        new
    }
*/

    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }


    
    pub fn len(&self) -> usize {
        self.len
    }
}


/*
impl<T, const N: usize> Default for Vec<T, {N}> {
    fn default() -> Self {
        Self::new()
    }
}
*/
impl<T, const N: usize> fmt::Debug for Vec<T, {N}>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <[T] as fmt::Debug>::fmt(self, f)
    }
}

impl<T, const N: usize> Drop for Vec<T, {N}> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut self[..]) }
    }
}

impl<T, const N: usize> Extend<T> for  Vec<T, {N}> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.extend(iter)
    }
}

impl<'a, T, const N: usize> Extend<&'a T> for Vec<T, {N}>
where
    T: 'a + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.extend(iter.into_iter().cloned())
    }
}

impl<'a, T, const N: usize> hash::Hash for Vec<T, {N}>
where
    T: core::hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        <[T] as hash::Hash>::hash(self, state)
    }
}

impl<'a, T, const N: usize> hash32::Hash for Vec<T, {N}>
where
    T: hash32::Hash,
{
    fn hash<H: hash32::Hasher>(&self, state: &mut H) {
        <[T] as hash32::Hash>::hash(self, state)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Vec<T, {N}> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Vec<T, {N}> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
/*
impl<T, const N: usize> FromIterator<T> for Vec<T, {N}> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = Vec::new();
        for i in iter {
            vec.push(i).ok().expect("Vec::from_iter overflow");
        }
        vec
    }
}*/

/// An iterator that moves out of an [`Vec`][`Vec`].
///
/// This struct is created by calling the `into_iter` method on [`Vec`][`Vec`].
///
/// [`Vec`]: (https://doc.rust-lang.org/std/vec/struct.Vec.html)
///
pub struct IntoIter<T, const N: usize> {
    vec: Vec<T, {N}>,
    next: usize,
}

impl<T, const N: usize> Iterator for IntoIter<T, {N}> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.vec.len() {
            let item = unsafe {
                (self.vec.buffer.as_ptr() as *const T)
                    .add(self.next)
                    .read()
            };
            self.next += 1;
            Some(item)
        } else {
            None
        }
    }
}
/*
impl<T, const N: usize> Clone for IntoIter<T, {N}>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            next: self.next,
        }
    }
}
*/
impl<T, const N: usize> Drop for IntoIter<T, {N}> {
    fn drop(&mut self) {
        unsafe {
            // Drop all the elements that have not been moved out of vec
            ptr::drop_in_place(&mut self.vec[self.next..]);
            // Prevent dropping of other elements
            self.vec.len = 0;
        }
    }
}

impl<T, const N: usize> IntoIterator for Vec<T, {N}> {
    type Item = T;
    type IntoIter = IntoIter<T, {N}>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { vec: self, next: 0 }
    }
}

impl<A, B, const N1: usize, const N2: usize> PartialEq<Vec<B, {N2}>> for Vec<A, {N1}>
where
    A: PartialEq<B>,
{
    fn eq(&self, other: &Vec<B, {N2}>) -> bool {
        <[A]>::eq(self, &**other)
    }
}

macro_rules! eq {
    ($Lhs:ty, $Rhs:ty) => {
        impl<'a, 'b, A, B, const N: usize> PartialEq<$Rhs> for $Lhs
        where
            A: PartialEq<B>,
        {
            fn eq(&self, other: &$Rhs) -> bool {
                <[A]>::eq(self, &other[..])
            }
        }
    };
}

eq!(Vec<A, {N}>, [B]);
eq!(Vec<A, {N}>, &'a [B]);
eq!(Vec<A, {N}>, &'a mut [B]);

eq!(Vec<A, {N}>, [B; {N}]);
eq!(Vec<A, {N}>, &'a [B; {N}]);

impl<T, const N: usize> Eq for Vec<T, {N}>
where
    T: Eq,
{
}

impl<T, const N: usize> ops::Deref for Vec<T, {N}> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> ops::DerefMut for Vec<T, {N}> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> AsRef<Vec<T, {N}>> for Vec<T, {N}> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, const N: usize> AsMut<Vec<T, {N}>> for Vec<T, {N}> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, const N: usize> AsRef<[T]> for Vec<T, {N}> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for Vec<T, {N}> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}


#[cfg(test)]
mod tests {
    use crate::Vec;

    #[test]
    fn static_new() {
        static mut _V: Vec<i32, 4> = Vec::new();
    }

    macro_rules! droppable {
        () => {
            struct Droppable;
            impl Droppable {
                fn new() -> Self {
                    unsafe {
                        COUNT += 1;
                    }
                    Droppable
                }
            }
            impl Drop for Droppable {
                fn drop(&mut self) {
                    unsafe {
                        COUNT -= 1;
                    }
                }
            }

            static mut COUNT: i32 = 0;
        };
    }

    #[test]
    fn drop() {
        droppable!();

        {
            let mut v: Vec<Droppable, 2> = Vec::new();
            v.push(Droppable::new()).ok().unwrap();
            v.push(Droppable::new()).ok().unwrap();
            v.pop().unwrap();
        }

        assert_eq!(unsafe { COUNT }, 0);

        {
            let mut v: Vec<Droppable, 2> = Vec::new();
            v.push(Droppable::new()).ok().unwrap();
            v.push(Droppable::new()).ok().unwrap();
        }

        assert_eq!(unsafe { COUNT }, 0);
    }

    #[test]
    fn eq() {
        let mut xs: Vec<i32, 4> = Vec::new();
        let mut ys: Vec<i32, 8> = Vec::new();

        assert_eq!(xs, ys);

        xs.push(1).unwrap();
        ys.push(1).unwrap();

        assert_eq!(xs, ys);
    }

    #[test]
    fn full() {
        let mut v: Vec<i32, 4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        assert!(v.push(4).is_err());
    }

    #[test]
    fn iter() {
        let mut v: Vec<i32, 4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        let mut items = v.iter();

        assert_eq!(items.next(), Some(&0));
        assert_eq!(items.next(), Some(&1));
        assert_eq!(items.next(), Some(&2));
        assert_eq!(items.next(), Some(&3));
        assert_eq!(items.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut v: Vec<i32, 4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        let mut items = v.iter_mut();

        assert_eq!(items.next(), Some(&mut 0));
        assert_eq!(items.next(), Some(&mut 1));
        assert_eq!(items.next(), Some(&mut 2));
        assert_eq!(items.next(), Some(&mut 3));
        assert_eq!(items.next(), None);
    }

    #[test]
    fn collect_from_iter() {
        let slice = &[1, 2, 3];
        let vec = slice.iter().cloned().collect::<Vec<_, 4>>();
        assert_eq!(vec, slice);
    }

    #[test]
    #[should_panic]
    fn collect_from_iter_overfull() {
        let slice = &[1, 2, 3];
        let _vec = slice.iter().cloned().collect::<Vec<_, 2>>();
    }

    #[test]
    fn iter_move() {
        let mut v: Vec<i32, 4> = Vec::new();
        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        let mut items = v.into_iter();

        assert_eq!(items.next(), Some(0));
        assert_eq!(items.next(), Some(1));
        assert_eq!(items.next(), Some(2));
        assert_eq!(items.next(), Some(3));
        assert_eq!(items.next(), None);
    }

    #[test]
    fn iter_move_drop() {
        droppable!();

        {
            let mut vec: Vec<Droppable, 2> = Vec::new();
            vec.push(Droppable::new()).ok().unwrap();
            vec.push(Droppable::new()).ok().unwrap();
            let mut items = vec.into_iter();
            // Move all
            let _ = items.next();
            let _ = items.next();
        }

        assert_eq!(unsafe { COUNT }, 0);

        {
            let mut vec: Vec<Droppable, 2> = Vec::new();
            vec.push(Droppable::new()).ok().unwrap();
            vec.push(Droppable::new()).ok().unwrap();
            let _items = vec.into_iter();
            // Move none
        }

        assert_eq!(unsafe { COUNT }, 0);

        {
            let mut vec: Vec<Droppable, 2> = Vec::new();
            vec.push(Droppable::new()).ok().unwrap();
            vec.push(Droppable::new()).ok().unwrap();
            let mut items = vec.into_iter();
            let _ = items.next(); // Move partly
        }

        assert_eq!(unsafe { COUNT }, 0);
    }

    #[test]
    fn push_and_pop() {
        let mut v: Vec<i32, 4> = Vec::new();
        assert_eq!(v.len(), 0);

        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);

        v.push(0).unwrap();
        assert_eq!(v.len(), 1);

        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.len(), 0);

        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn resize_size_limit() {
        let mut v: Vec<u8, 4> = Vec::new();

        v.resize(0, 0).unwrap();
        v.resize(4, 0).unwrap();
        v.resize(5, 0).err().expect("full");
    }

    #[test]
    fn resize_length_cases() {
        let mut v: Vec<u8, 4> = Vec::new();

        assert_eq!(v.len(), 0);

        // Grow by 1
        v.resize(1, 0).unwrap();
        assert_eq!(v.len(), 1);

        // Grow by 2
        v.resize(3, 0).unwrap();
        assert_eq!(v.len(), 3);

        // Resize to current size
        v.resize(3, 0).unwrap();
        assert_eq!(v.len(), 3);

        // Shrink by 1
        v.resize(2, 0).unwrap();
        assert_eq!(v.len(), 2);

        // Shrink by 2
        v.resize(0, 0).unwrap();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn resize_contents() {
        let mut v: Vec<u8, 4> = Vec::new();

        // New entries take supplied value when growing
        v.resize(1, 17).unwrap();
        assert_eq!(v[0], 17);

        // Old values aren't changed when growing
        v.resize(2, 18).unwrap();
        assert_eq!(v[0], 17);
        assert_eq!(v[1], 18);

        // Old values aren't changed when length unchanged
        v.resize(2, 0).unwrap();
        assert_eq!(v[0], 17);
        assert_eq!(v[1], 18);

        // Old values aren't changed when shrinking
        v.resize(1, 0).unwrap();
        assert_eq!(v[0], 17);
    }

    #[test]
    fn resize_default() {
        let mut v: Vec<u8, 4> = Vec::new();

        // resize_default is implemented using resize, so just check the
        // correct value is being written.
        v.resize_default(1).unwrap();
        assert_eq!(v[0], 0);
    }
}
