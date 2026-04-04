//! Thread safe implementations of utilities.

use ::parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Setting, SettingsError, cached::Guard, io::SettingsStore, sync::backend::Sync};

mod backend {
    //! Guard backend.

    use ::parking_lot::MappedRwLockReadGuard;

    use crate::cached::Backend;

    /// Sync backend.
    #[derive(Debug)]
    pub enum Sync {}

    impl Backend for Sync {
        type Ref<'a, T: 'a + ?Sized> = MappedRwLockReadGuard<'a, T>;

        fn map<'a, T, F, U>(this: Self::Ref<'a, T>, f: F) -> Self::Ref<'a, U>
        where
            F: FnOnce(&T) -> &U,
            U: ?Sized,
            T: ?Sized,
        {
            MappedRwLockReadGuard::map(this, f)
        }
    }
}

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
    ) -> Result<Guard<'this, T, Sync>, SettingsError> {
        let guard = self.cache.read();

        if let Ok(referenced) =
            RwLockReadGuard::try_map_or_err(guard, |inner| inner.as_ref().ok_or(()))
            && self.generation == store.generation()
        {
            return Ok(Guard::new(referenced));
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

        Ok(Guard::new(referenced))
    }
}
