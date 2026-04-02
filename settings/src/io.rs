//! Reading and writing of settings.

use crate::{Primitive, SettingsError};

/// Trait used to perform settings read and write operations.
pub trait SettingsStore {
    /// Read a setting.
    ///
    /// # Errors
    /// If the setting cannot be read, missing settings should not return an error, instead they
    /// should return [Primitive::Null].
    fn read_setting(&self, path: &str) -> Result<Primitive, SettingsError>;

    /// Write a setting.
    ///
    /// # Errors
    /// If the setting cannot be written to store.
    fn write_setting(&mut self, path: &str, value: Primitive) -> Result<(), SettingsError>;

    /// What settings generation path in store is currently on.
    /// Should be changed on writes to a value not used previously.
    ///
    /// It is perfectly valid to increase it on any write, as it's intended use is cache
    /// invalidation.
    fn generation(&self) -> u64;
}
