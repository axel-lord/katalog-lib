//! Common proc macro utilities

pub mod err_collector {
    //! Utilities to collect either errors or valid values.

    /// Collect errors into collection C of errors E or valid values into T.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct ErrCollector<C, T> {
        /// Valid value storage.
        value: T,
        /// Error collector.
        err: C,
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
