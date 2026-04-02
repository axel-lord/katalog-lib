//! Cache settings results.

use ::core::{
    cell::{Ref, RefCell},
    ops::Deref,
};
use ::std::borrow::Cow;

use crate::{Setting, SettingsError, io::SettingsStore};

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
    ) -> Result<Guard<'this, T>, SettingsError> {
        if let Ok(referenced) = Ref::filter_map(self.cache.try_borrow()?, |inner| inner.as_ref())
            && self.generation == store.generation()
        {
            return Ok(Guard { referenced });
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

        Ok(Guard { referenced })
    }
}

/// Guard for borrowed settings value.
#[derive(Debug)]
pub struct Guard<'a, T: ?Sized> {
    /// Wrapped ref.
    referenced: Ref<'a, T>,
}

impl<'a, T: ?Sized> Guard<'a, T> {
    /// Clone the guard.
    ///
    /// Note: Is an associated function in order
    /// to not conflict with clone impl of guarded value.
    #[expect(
        clippy::should_implement_trait,
        reason = "follows same pattern as wrapped ref"
    )]
    pub fn clone(orig: &Guard<'a, T>) -> Self {
        Guard {
            referenced: Ref::clone(&orig.referenced),
        }
    }

    /// Get guard to type for which `T` implements [AsRef].
    pub fn as_ref<R>(this: Guard<'a, T>) -> Guard<'a, R>
    where
        T: AsRef<R>,
    {
        Guard {
            referenced: Ref::map(this.referenced, T::as_ref),
        }
    }

    /// Get guard to type for which `T` implements [Deref].
    pub fn as_deref(this: Guard<'a, T>) -> Guard<'a, T::Target>
    where
        T: Deref,
    {
        Guard {
            referenced: Ref::map(this.referenced, T::deref),
        }
    }
}

impl<'a, T: ?Sized> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.referenced
    }
}
