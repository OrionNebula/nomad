use std::borrow::Borrow;
use std::marker::PhantomData;
use std::slice;

pub struct OrderedArray<T, C: AsRef<[T]>>(C, PhantomData<T>)
where
    T: Ord;

impl<T, C: AsRef<[T]>> OrderedArray<T, C>
where
    T: Ord,
{
    // Create a new OrderedArray, assuming the container is already ordered
    pub unsafe fn new_unsafe(container: C) -> Self {
        OrderedArray(container, PhantomData)
    }

    // Attempt to create a new OrderedArray from a container.
    // Returns None if the container is not already sorted.
    pub fn try_new(container: C) -> Option<Self> {
        let mut last_item: Option<&T> = None;

        for item in container.as_ref() {
            if let Some(last_item) = last_item {
                if last_item >= item {
                    return None;
                }
            }

            last_item = Some(item);
        }

        Some(OrderedArray(container, PhantomData))
    }

    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.0.as_ref().iter()
    }
}

// Allow conversion of a borrow to an iterator
impl<'a, T: Ord, C: AsRef<[T]>> IntoIterator for &'a OrderedArray<T, C> {
    type IntoIter = slice::Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_ref().iter()
    }
}

// Allow borrowing of the internal slice
impl<T: Ord, C: AsRef<[T]>> AsRef<[T]> for OrderedArray<T, C> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

// Allow borrowing of the internal slice
impl<T: Ord, C: AsRef<[T]>> Borrow<[T]> for OrderedArray<T, C> {
    fn borrow(&self) -> &[T] {
        self.0.as_ref()
    }
}

// Allow conversion of a borrowed slice-like to a owned slice-like without copying
// Transposes the reference from the outside to the inside
impl<'a, T: Ord, C: AsRef<[T]>> From<&'a OrderedArray<T, C>>
    for OrderedArray<T, &'a OrderedArray<T, C>>
{
    fn from(slice_ref: &'a OrderedArray<T, C>) -> Self {
        OrderedArray(slice_ref, PhantomData)
    }
}

// Allow the creation of an ordered slice-like
impl<'a, T, C> From<&'a C> for OrderedArray<T, Vec<T>>
where
    T: Ord + Clone,
    &'a C: Iterator<Item = T>,
{
    fn from(container: &'a C) -> Self {
        let mut vec: Vec<T> = container.collect();
        vec.sort_unstable();

        OrderedArray(vec, PhantomData)
    }
}

impl<I: Ord, C: AsRef<[I]> + AsMut<[I]>> From<C> for OrderedArray<I, C> {
    fn from(mut container: C) -> Self {
        let slice = container.as_mut();
        slice.sort_unstable();

        OrderedArray(container, PhantomData)
    }
}
