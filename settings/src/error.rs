//! [SettingsError] impl.

use ::core::{
    error::Error,
    fmt::{Debug, Display},
};

/// Error type returned by settings handling.
pub struct SettingsError {
    /// Wrapped error value.
    err: Box<dyn Error + Send + Sync>,
}

impl Debug for SettingsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let Self { err } = self;
        Debug::fmt(err, f)
    }
}

impl Display for SettingsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        Display::fmt(&self.err, f)
    }
}

/// An unknown settings error.
#[derive(Debug)]
pub struct UnknownSettingsError;

/// A list of errors with a parent.
#[derive(Debug)]
pub struct ErrStack {
    /// Top-most error.
    pub parent: SettingsError,
    /// Error stack with last element being
    /// most recent.
    pub errors: Vec<SettingsError>,
}

impl ErrStack {
    /// Run a closure for entire error stack, including close nesting.
    fn try_for_each<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: for<'a> FnMut(&'a SettingsError) -> Result<(), E>,
    {
        use ::core::iter::once;
        let Self { parent, errors } = self;

        let mut stack_stack = vec![once(parent).chain(errors.iter().rev())];

        loop {
            let Some(tail) = stack_stack.last_mut() else {
                break;
            };

            let Some(err) = tail.next() else {
                stack_stack.pop();
                continue;
            };

            if let Some(ErrStack { parent, errors }) = err.err.downcast_ref() {
                stack_stack.push(once(parent).chain(errors.iter().rev()));
                continue;
            }

            f(err)?
        }

        Ok(())
    }
}

impl Display for ErrStack {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        self.try_for_each(|err| Display::fmt(err, f))
    }
}

impl Error for ErrStack {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.parent.err)
    }
}

impl Display for UnknownSettingsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str("unknown settings error")
    }
}
impl Error for UnknownSettingsError {}

/// Implementation for [SettingsError::wrapped].
fn wrapped_settings_error(msg: SettingsError, err: SettingsError) -> SettingsError {
    match [
        msg.err.downcast::<ErrStack>(),
        err.err.downcast::<ErrStack>(),
    ] {
        [Ok(msg_stack), Ok(mut err_stack)] => {
            let ErrStack { mut parent, errors } = *msg_stack;
            ::core::mem::swap(&mut parent, &mut err_stack.parent);
            err_stack.errors.push(parent);
            err_stack.errors.extend(errors);
            err_stack.into()
        }
        [Ok(mut msg_stack), Err(err)] => {
            msg_stack.errors.insert(0, err.into());
            msg_stack.into()
        }
        [Err(mut msg), Ok(mut err_stack)] => {
            ::core::mem::swap(&mut msg, &mut err_stack.parent.err);
            err_stack.errors.push(msg.into());
            err_stack.into()
        }
        [Err(msg), Err(err)] => ErrStack {
            parent: msg.into(),
            errors: Vec::from_iter([err.into()]),
        }
        .into(),
    }
}

impl SettingsError {
    /// Create an unknown error.
    pub fn unknown() -> Self {
        Self {
            err: Box::new(UnknownSettingsError),
        }
    }

    /// Wrap an error with a message.
    pub fn wrapped(msg: impl Into<SettingsError>, err: impl Into<SettingsError>) -> Self {
        wrapped_settings_error(msg.into(), err.into())
    }

    /// Get reference to underlying error, if any.
    pub fn by_ref(&self) -> &(dyn Error + Send + Sync) {
        &*self.err
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
                Some(&*self.0.err)
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
        SettingsError { err: value.into() }
    }
}
