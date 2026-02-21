//! Utilities to collect either errors or valid values.

use ::core::ops::{Deref, DerefMut};

/// Collect errors into collection C of errors E or valid values into T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ErrCollector<ErrCollection, ValueTy = ()> {
    /// Valid value storage.
    pub value: ValueTy,
    /// Error collector.
    pub err: ErrCollection,
}

impl<ErrCollection, ValueTy> ErrCollector<ErrCollection, ValueTy> {
    /// Push an error to error collection.
    pub fn push_err<Err>(&mut self, err: Err) -> &mut Self
    where
        ErrCollection: Extend<Err>,
    {
        self.err.extend([err]);
        self
    }

    /// Extend error collection with values of err.
    pub fn extend_err<IntoIter>(&mut self, err: IntoIter) -> &mut Self
    where
        IntoIter: IntoIterator,
        ErrCollection: Extend<IntoIter::Item>,
    {
        self.err.extend(err);
        self
    }

    /// Split into value and error collection without a value.
    pub fn split(self) -> (ValueTy, ErrCollector<ErrCollection, ()>) {
        let Self { value, err } = self;
        (value, ErrCollector { value: (), err })
    }

    /// Replace current value with `value`.
    pub fn with<ValueTyB>(self, value: ValueTyB) -> ErrCollector<ErrCollection, ValueTyB> {
        let Self { value: _, err } = self;
        ErrCollector { value, err }
    }

    /// Convert a result with a single error into an error collection.
    pub fn from_result<Err>(result: Result<ValueTy, Err>) -> Self
    where
        ErrCollection: FromIterator<Err> + Default,
        ValueTy: Default,
    {
        match result {
            Ok(value) => Self {
                value,
                err: ErrCollection::default(),
            },
            Err(e) => Self {
                value: ValueTy::default(),
                err: ErrCollection::from_iter([e]),
            },
        }
    }

    /// Convert into a result with an error type which may be extended by itself.
    ///
    /// # Errors
    /// If err contains any errors.
    pub fn into_result<Err>(self) -> Result<ValueTy, Err>
    where
        ErrCollection: IntoIterator,
        Err: From<ErrCollection::Item> + Extend<Err>,
    {
        let Self { value, err } = self;
        let mut err_iter = err.into_iter().map(Err::from);

        if let Some(mut err) = err_iter.next() {
            err.extend(err_iter);
            Err(err)
        } else {
            Ok(value)
        }
    }

    /// Collect iterator of results into a collection and self.
    pub fn collect<Collection>(
        &mut self,
        into_iter: impl IntoIterator<Item = Result<Collection::Item, ErrCollection::Item>>,
    ) -> Collection
    where
        Collection: IntoIterator + FromIterator<Collection::Item>,
        ErrCollection: IntoIterator + Extend<ErrCollection::Item>,
    {
        into_iter
            .into_iter()
            .filter_map(|result| match result {
                Ok(value) => Some(value),
                Err(err) => {
                    self.push_err(err);
                    None
                }
            })
            .collect()
    }

    /// Run scope function converting result into an option and adding the
    /// error to collection if any.
    pub fn scope<Scope, T, E>(&mut self, scope: Scope) -> Option<T>
    where
        Scope: FnOnce() -> Result<T, E>,
        ErrCollection: Extend<E>,
    {
        match scope() {
            Ok(value) => Some(value),
            Err(err) => {
                self.push_err(err);
                None
            }
        }
    }
}

impl<E, T> Deref for ErrCollector<E, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<E, T> DerefMut for ErrCollector<E, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<E, T> AsRef<T> for ErrCollector<E, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<E, T> AsMut<T> for ErrCollector<E, T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<E, T> IntoIterator for ErrCollector<E, T>
where
    T: IntoIterator,
{
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        T::into_iter(self.value)
    }
}

impl<'i, E, T> IntoIterator for &'i ErrCollector<E, T>
where
    &'i T: IntoIterator,
{
    type Item = <&'i T as IntoIterator>::Item;
    type IntoIter = <&'i T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <&'i T as IntoIterator>::into_iter(&self.value)
    }
}

impl<'i, E, T> IntoIterator for &'i mut ErrCollector<E, T>
where
    &'i mut T: IntoIterator,
{
    type Item = <&'i mut T as IntoIterator>::Item;

    type IntoIter = <&'i mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <&'i mut T as IntoIterator>::into_iter(&mut self.value)
    }
}

impl<ErrCollection, ValueTy, Err, ItemTy> FromIterator<Result<ItemTy, Err>>
    for ErrCollector<ErrCollection, ValueTy>
where
    ValueTy: FromIterator<ItemTy>,
    ErrCollection: Default + Extend<Err>,
{
    fn from_iter<I: IntoIterator<Item = Result<ItemTy, Err>>>(iter: I) -> Self {
        let mut err = ErrCollection::default();

        let value = ValueTy::from_iter(iter.into_iter().filter_map(|result| match result {
            Ok(v) => Some(v),
            Err(e) => {
                err.extend([e]);
                None
            }
        }));

        Self { value, err }
    }
}
