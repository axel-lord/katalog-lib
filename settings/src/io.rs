//! Reading and writing of settings.

use crate::{Primitive, SettingsError};

#[cfg(feature = "provider")]
pub use provider::IoProvider;

/// Trait used to perform settings read and write operations.
pub trait SettingsIo {
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
}

#[cfg(feature = "provider")]
mod provider {
    //! [IoProvider] trait.
    use ::bytemuck::TransparentWrapper;

    use crate::{Primitive, SettingsError, io::SettingsIo};

    /// Provide [SettingsIo] implementations to [IoProvider] impls.
    #[derive(TransparentWrapper)]
    #[repr(transparent)]
    struct IoProviderStoreWrapper<S: IoProvider + ?Sized>(S::Store);

    impl<S: IoProvider + ?Sized> SettingsIo for IoProviderStoreWrapper<S> {
        fn read_setting(&self, path: &str) -> Result<Primitive, SettingsError> {
            let Self(store) = self;
            S::read_setting(store, path)
        }

        fn write_setting(&mut self, path: &str, value: Primitive) -> Result<(), SettingsError> {
            let Self(store) = self;
            S::write_setting(store, path, value)
        }
    }

    /// Trait used to ensure settings operations are available
    /// for a type. Used to provide [SettingsIo] implementations
    /// for foreign types.
    pub trait IoProvider: 'static {
        /// Settings store to use.
        type Store;

        /// Read a setting from store.
        ///
        /// # Errors
        /// If the setting cannot be read, missing settings should not return an error, isntead they
        /// should return [Primitive::Null].
        fn read_setting(store: &Self::Store, path: &str) -> Result<Primitive, SettingsError>;

        /// Write a setting to store.
        ///
        /// # Errors
        /// If the setting cannot be written to store.
        fn write_setting(
            store: &mut Self::Store,
            path: &str,
            value: Primitive,
        ) -> Result<(), SettingsError>;

        /// Provide a settings io implementation with read andwrite access.
        fn settings_io_mut(store: &mut Self::Store) -> &mut dyn SettingsIo {
            IoProviderStoreWrapper::<Self>::wrap_mut(store)
        }

        /// Provide a sync settings io implementation with read andwrite access.
        fn settings_io_sync_mut(store: &mut Self::Store) -> &mut (dyn Sync + SettingsIo)
        where
            Self::Store: Sync,
        {
            IoProviderStoreWrapper::<Self>::wrap_mut(store)
        }

        /// Provide a settings io implementation with read andwrite access.
        fn settings_io(store: &Self::Store) -> &dyn SettingsIo {
            IoProviderStoreWrapper::<Self>::wrap_ref(store)
        }

        /// Provide a sync settings io implementation with read andwrite access.
        fn settings_io_sync(store: &Self::Store) -> &(dyn Sync + SettingsIo)
        where
            Self::Store: Sync,
        {
            IoProviderStoreWrapper::<Self>::wrap_ref(store)
        }
    }
}
