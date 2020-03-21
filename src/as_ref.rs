use std::ops::{Deref, Index};

/// Wraps an `AsRef` type to slice the result of `as_ref`.
pub struct ReindexAsRef<T, I: Clone>(pub T, pub I);

impl<T, I, R> AsRef<R> for ReindexAsRef<T, I>
where
    T: AsRef<R>,
    I: Clone,
    R: Index<I, Output = R> + ?Sized,
{
    fn as_ref(&self) -> &R {
        &self.0.as_ref()[self.1.clone()]
    }
}

/// Forwards a `Deref` type's `AsRef` to the `AsRef` of the deref'd type.
pub struct ForwardAsRef<T: Deref>(pub T);

impl<T, R> AsRef<R> for ForwardAsRef<T>
where
    T: Deref,
    T::Target: AsRef<R>,
    R: ?Sized,
{
    fn as_ref(&self) -> &R {
        self.0.deref().as_ref()
    }
}
