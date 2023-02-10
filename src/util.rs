use std::marker::PhantomData;

pub struct OkIter<T, E, I> {
    inner: I,
    error: Option<E>,
    _type: PhantomData<T>,
}

impl<T, E, I> OkIter<T, E, I> {
    pub const fn new(inner: I) -> Self {
        Self {
            inner,
            error: None,
            _type: PhantomData,
        }
    }

    pub const fn to_error(&self) -> Option<&E> {
        self.error.as_ref()
    }
}

impl<T, E, I> Iterator for OkIter<T, E, I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_none() {
            match self.inner.next()? {
                Ok(v) => Some(v),
                Err(e) => {
                    self.error = Some(e);
                    None
                }
            }
        } else {
            None
        }
    }
}
