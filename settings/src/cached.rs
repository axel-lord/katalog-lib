//! Cache settings results.

use ::core::{
    cell::{Ref, RefCell},
    ops::Deref,
};
use ::std::borrow::Cow;

use crate::{RefSetting, Setting, SettingsError, io::SettingsStore};

/// A cached settings value, will use existing
/// value on generation match otherwise retrieve new value.
#[derive(Debug)]
pub struct Cached<'lt, S: 'lt, T> {
    /// Setting to use for value retrieval.
    setting: &'lt S,
    /// Cached setting.
    cache: RefCell<Option<T>>,
    /// Generation of setting.
    generation: u64,
}

impl<'lt, S: 'lt, T: 'static> Cached<'lt, S, T>
where
    S: AsRef<Setting<T>>,
{
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

        let setting = self.setting.as_ref();
        let primitive = store.read_setting(setting.path())?;
        let value = setting.try_from_primitive(primitive)?;
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

impl<'lt, T, R> Cached<'lt, RefSetting<T, R>, T> {}

/// Guard for borrowed settings value.
#[derive(Debug)]
pub struct Guard<'a, T> {
    /// Wrapped ref.
    referenced: Ref<'a, T>,
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.referenced
    }
}
