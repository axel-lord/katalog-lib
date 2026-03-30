//! [SettingsError] impl.

use ::core::{
    error::Error,
    fmt::{Debug, Display},
};

/// Error type returned by settings handling.
pub struct SettingsError {
    /// Wrapped error value.
    err: Option<Box<dyn Error + Send + Sync>>,
}

impl Debug for SettingsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let Self { err } = self;
        Debug::fmt(err, f)
    }
}

impl Display for SettingsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        if let Some(err) = &self.err {
            Display::fmt(err, f)
        } else {
            f.write_str("unknown settings error")
        }
    }
}

impl SettingsError {
    /// Create an unknown error.
    pub const fn unknown() -> Self {
        Self { err: None }
    }

    /// Get reference to underlying error, if any.
    pub fn by_ref(&self) -> Option<&(dyn Error + Send + Sync)> {
        if let Some(err) = &self.err {
            Some(err.as_ref())
        } else {
            None
        }
    }

    /// Convert into a type implementing [Error].
    ///
    /// Implementation provides source impl returning source when
    /// available.
    pub fn into_error(self) -> impl Error + Send + Sync {
        #[repr(transparent)]
        struct Wrap(SettingsError);
        impl Debug for Wrap {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let Wrap(inner) = self;
                Debug::fmt(inner, f)
            }
        }
        impl Display for Wrap {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let Wrap(inner) = self;
                Display::fmt(inner, f)
            }
        }
        impl Error for Wrap {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                if let Some(err) = &self.0.err {
                    Some(err.as_ref())
                } else {
                    None
                }
            }
        }
        Wrap(self)
    }
}

impl<E> From<E> for SettingsError
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn from(value: E) -> Self {
        SettingsError {
            err: Some(value.into()),
        }
    }
}
