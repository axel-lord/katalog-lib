//! Common proc macro utilities

pub mod err_collector {
    //! Utilities to collect either errors or valid values.

    use ::core::ops::{Deref, DerefMut};

    /// Collect errors into collection C of errors E or valid values into T.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct ErrCollector<C, T> {
        /// Valid value storage.
        pub value: T,
        /// Error collector.
        pub err: C,
    }

    impl<C, T> ErrCollector<C, T> {
        /// Push an error to error collection.
        pub fn push_err<E>(&mut self, err: E) -> &mut Self
        where
            C: Extend<E>,
        {
            self.err.extend([err]);
            self
        }

        /// Extend error collection with values of err.
        pub fn extend_err<I>(&mut self, err: I) -> &mut Self
        where
            I: IntoIterator,
            C: Extend<I::Item>,
        {
            self.err.extend(err);
            self
        }

        /// Split into value and error collection without a value.
        pub fn split(self) -> (T, ErrCollector<C, ()>) {
            let Self { value, err } = self;
            (value, ErrCollector { value: (), err })
        }

        /// Replace current value with `value`.
        pub fn with<V>(self, value: V) -> ErrCollector<C, V> {
            let Self { value: _, err } = self;
            ErrCollector { value, err }
        }

        /// Convert a result with a single error into an error collection.
        pub fn from_result<E>(result: Result<T, E>) -> Self
        where
            C: FromIterator<E> + Default,
            T: Default,
        {
            match result {
                Ok(value) => Self {
                    value,
                    err: C::default(),
                },
                Err(e) => Self {
                    value: T::default(),
                    err: C::from_iter([e]),
                },
            }
        }

        /// Convert into a result with an error type which may be extended by itself.
        ///
        /// # Errors
        /// If err contains any errors.
        pub fn into_result<E>(self) -> Result<T, E>
        where
            C: IntoIterator,
            E: From<C::Item> + Extend<E>,
        {
            let Self { value, err } = self;
            let mut err_iter = err.into_iter().map(E::from);

            if let Some(mut err) = err_iter.next() {
                err.extend(err_iter);
                Err(err)
            } else {
                Ok(value)
            }
        }
    }

    impl<C, T> Deref for ErrCollector<C, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    impl<C, T> DerefMut for ErrCollector<C, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.value
        }
    }

    impl<C, T> AsRef<T> for ErrCollector<C, T> {
        fn as_ref(&self) -> &T {
            self
        }
    }

    impl<C, T> AsMut<T> for ErrCollector<C, T> {
        fn as_mut(&mut self) -> &mut T {
            self
        }
    }

    impl<C, T> IntoIterator for ErrCollector<C, T>
    where
        T: IntoIterator,
    {
        type Item = T::Item;

        type IntoIter = T::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            T::into_iter(self.value)
        }
    }

    impl<'i, C, T> IntoIterator for &'i ErrCollector<C, T>
    where
        &'i T: IntoIterator,
    {
        type Item = <&'i T as IntoIterator>::Item;
        type IntoIter = <&'i T as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            <&'i T as IntoIterator>::into_iter(&self.value)
        }
    }

    impl<'i, C, T> IntoIterator for &'i mut ErrCollector<C, T>
    where
        &'i mut T: IntoIterator,
    {
        type Item = <&'i mut T as IntoIterator>::Item;

        type IntoIter = <&'i mut T as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            <&'i mut T as IntoIterator>::into_iter(&mut self.value)
        }
    }

    impl<C, T, E, V> FromIterator<Result<V, E>> for ErrCollector<C, T>
    where
        T: FromIterator<V>,
        C: Default + Extend<E>,
    {
        fn from_iter<I: IntoIterator<Item = Result<V, E>>>(iter: I) -> Self {
            let mut err = C::default();

            let value = T::from_iter(iter.into_iter().filter_map(|result| match result {
                Ok(v) => Some(v),
                Err(e) => {
                    err.extend([e]);
                    None
                }
            }));

            Self { value, err }
        }
    }
}
