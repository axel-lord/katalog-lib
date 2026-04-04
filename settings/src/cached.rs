//! Cache settings results.

use ::core::{
    cell::{Ref, RefCell},
    ops::Deref,
};
use ::std::borrow::Cow;

use crate::{Setting, SettingsError, cached::backend::Unsync, io::SettingsStore};

pub(crate) use crate::cached::backend::Backend;

mod backend {
    //! Guard backend.

    use ::core::{cell::Ref, ops::Deref};

    /// Backend to a cached guard.
    pub trait Backend {
        /// Reference type of backend.
        type Ref<'a, T: 'a + ?Sized>: Deref<Target = T>;

        /// Map guarded value.
        fn map<'a, T, F, U>(this: Self::Ref<'a, T>, f: F) -> Self::Ref<'a, U>
        where
            F: FnOnce(&T) -> &U,
            U: ?Sized,
            T: ?Sized;
    }

    /// Unsync backend
    #[derive(Debug)]
    pub enum Unsync {}

    impl Backend for Unsync {
        type Ref<'a, T: 'a + ?Sized> = Ref<'a, T>;

        fn map<'a, T, F, U>(this: Self::Ref<'a, T>, f: F) -> Self::Ref<'a, U>
        where
            F: FnOnce(&T) -> &U,
            U: ?Sized,
            T: ?Sized,
        {
            Ref::map(this, f)
        }
    }
}

/// A cached settings value, will use existing
/// value on generation match otherwise retrieve new value.
#[derive(Debug)]
pub struct Cached<'lt, T: 'static> {
    /// Setting to use for value retrieval.
    setting: &'lt Setting<T>,
    /// Cached setting.
    cache: RefCell<Option<T>>,
    /// Generation of setting.
    generation: u64,
}

impl<'lt, T: 'static> Clone for Cached<'lt, T> {
    fn clone(&self) -> Self {
        Self::new(self.setting)
    }
}

impl<'lt, T: 'static> Cached<'lt, T> {
    /// Construct a new cached setting.
    pub const fn new(setting: &'lt Setting<T>) -> Self {
        Self {
            setting,
            cache: RefCell::new(None),
            generation: 0,
        }
    }

    /// Get settings value from store.
    ///
    /// # Errors
    /// If the value cannot be retrieved.
    pub fn get<'this>(
        &'this self,
        store: &dyn SettingsStore,
    ) -> Result<Guard<'this, T, Unsync>, SettingsError> {
        if let Ok(referenced) = Ref::filter_map(self.cache.try_borrow()?, |inner| inner.as_ref())
            && self.generation == store.generation()
        {
            return Ok(Guard::new(referenced));
        }

        let primitive = store.read_setting(self.setting.path())?;
        let value = self.setting.try_from_primitive(primitive)?;
        {
            let mut guard = self.cache.try_borrow_mut().map_err(|err| {
                SettingsError::wrapped(
                    Cow::Borrowed(
                        "setting updated whilst borrowed\
                   A\nntip: in the same context clone \
                   the guard of a single borrow instead \
                   of calling Cached::get multiple times",
                    ),
                    err,
                )
            })?;
            *guard = Some(value);
        }
        let referenced = Ref::filter_map(self.cache.try_borrow()?, |inner| inner.as_ref())
            .map_err(|_| "cached value was not set, should not happen")?;

        Ok(Guard::new(referenced))
    }
}

/// Guard for borrowed settings value.
#[derive(Debug)]
pub struct Guard<'a, T: 'a + ?Sized, B: Backend> {
    /// Wrapped ref.
    referenced: B::Ref<'a, T>,
}

impl<'a, T: 'a + ?Sized> Guard<'a, T, Unsync> {
    /// Clone the guard.
    ///
    /// Note: Is an associated function in order
    /// to not conflict with clone impl of guarded value.
    #[expect(
        clippy::should_implement_trait,
        reason = "follows same pattern as wrapped ref"
    )]
    pub fn clone(orig: &Guard<'a, T, Unsync>) -> Self {
        Guard {
            referenced: Ref::clone(&orig.referenced),
        }
    }
}

impl<'a, T: 'a + ?Sized, B: Backend> Guard<'a, T, B> {
    /// Create a new guard.
    pub(crate) const fn new(referenced: B::Ref<'a, T>) -> Self {
        Self { referenced }
    }

    /// Map guarded value.
    pub fn map<F, U>(this: Guard<'a, T, B>, f: F) -> Guard<'a, U, B>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        let Guard { referenced } = this;
        let referenced = B::map(referenced, f);
        Guard { referenced }
    }

    /// Get guard to type for which `T` implements [AsRef].
    pub fn as_ref<R>(this: Guard<'a, T, B>) -> Guard<'a, R, B>
    where
        T: AsRef<R>,
    {
        Guard::map(this, T::as_ref)
    }

    /// Get guard to type for which `T` implements [Deref].
    pub fn as_deref(this: Guard<'a, T, B>) -> Guard<'a, T::Target, B>
    where
        T: Deref,
    {
        Guard::map(this, T::deref)
    }
}

impl<'a, T: ?Sized, B: Backend> Deref for Guard<'a, T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.referenced
    }
}
