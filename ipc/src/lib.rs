//! Ipc utilities.

pub mod single_process;
mod static_path;

pub use ::iceoryx2::prelude::ZeroCopySend;
pub use ::iceoryx2_bb_container as container;
pub use single_process::single_process;
pub use static_path::{FromPathError, IntoPathError, StaticPath};
