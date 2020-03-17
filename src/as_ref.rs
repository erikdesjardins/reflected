use std::marker::PhantomData;
use std::ops::{Deref, Index};

/// Wraps an `AsRef` type to slice the result of `as_ref`.
pub struct ReindexAsRef<T, I, R>
where
    T: AsRef<R>,
    I: Clone,
    R: Index<I> + ?Sized,
{
    value: T,
    index: I,
    _ref: PhantomData<R>,
}

impl<T, I, R> ReindexAsRef<T, I, R>
where
    T: AsRef<R>,
    I: Clone,
    R: Index<I> + ?Sized,
{
    pub fn new(value: T, index: I) -> Self {
        Self {
            value,
            index,
            _ref: PhantomData,
        }
    }
}

impl<T, I, R> AsRef<R::Output> for ReindexAsRef<T, I, R>
where
    T: AsRef<R>,
    I: Clone,
    R: Index<I> + ?Sized,
{
    fn as_ref(&self) -> &R::Output {
        &self.value.as_ref()[self.index.clone()]
    }
}

/// Forwards a `Deref` type's `AsRef` to the `AsRef` of the deref'd type.
pub struct ForwardAsRef<T, R>
where
    T: Deref,
    T::Target: AsRef<R>,
    R: ?Sized,
{
    value: T,
    _ref: PhantomData<R>,
}

impl<T, R> ForwardAsRef<T, R>
where
    T: Deref,
    T::Target: AsRef<R>,
    R: ?Sized,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            _ref: PhantomData,
        }
    }
}

impl<T, R> AsRef<R> for ForwardAsRef<T, R>
where
    T: Deref,
    T::Target: AsRef<R>,
    R: ?Sized,
{
    fn as_ref(&self) -> &R {
        self.value.deref().as_ref()
    }
}
