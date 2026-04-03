//! Thread safe implementations of utilities.

use ::core::ops::Deref;

use ::parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Setting, SettingsError, io::SettingsStore};

/// A cached settings value, will use existing
/// value on generation match otherwise retrieve new value.
///
/// Sync implementation of [Cached][crate::cached::Cached].
#[derive(Debug)]
pub struct Cached<'lt, T: 'static> {
    /// Setting to use for value retrieval.
    setting: &'lt Setting<T>,
    /// Cached setting.
    cache: RwLock<Option<T>>,
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
            cache: RwLock::new(None),
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
        let guard = self.cache.read();

        if let Ok(referenced) =
            RwLockReadGuard::try_map_or_err(guard, |inner| inner.as_ref().ok_or(()))
            && self.generation == store.generation()
        {
            return Ok(Guard { referenced });
        }

        let primitive = store.read_setting(self.setting.path())?;
        let value = self.setting.try_from_primitive(primitive)?;
        let mut guard = self.cache.write();
        *guard = Some(value);

        let referenced =
            RwLockReadGuard::try_map_or_err(RwLockWriteGuard::downgrade(guard), |inner| {
                inner.as_ref().ok_or(())
            })
            .map_err(|_| "cached value not set, should not happen")?;

        Ok(Guard { referenced })
    }
}

/// Guard for borrowed settings value.
#[derive(Debug)]
pub struct Guard<'a, T: ?Sized> {
    /// Wrapped ref.
    referenced: MappedRwLockReadGuard<'a, T>,
}

impl<'a, T: ?Sized> Guard<'a, T> {
    /// Map guarded value.
    pub fn map<F, U>(this: Guard<'a, T>, f: F) -> Guard<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        let Guard { referenced } = this;
        let referenced = MappedRwLockReadGuard::map(referenced, f);
        Guard { referenced }
    }

    /// Get guard to type for which `T` implements [AsRef].
    pub fn as_ref<R>(this: Guard<'a, T>) -> Guard<'a, R>
    where
        T: AsRef<R>,
    {
        Guard::map(this, T::as_ref)
    }

    /// Get guard to type for which `T` implements [Deref].
    pub fn as_deref(this: Guard<'a, T>) -> Guard<'a, T::Target>
    where
        T: Deref,
    {
        Guard::map(this, T::deref)
    }
}

impl<'a, T: ?Sized> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.referenced
    }
}
