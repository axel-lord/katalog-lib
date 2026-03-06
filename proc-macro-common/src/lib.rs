//! Common proc macro utilities
#![allow(missing_debug_implementations, reason = "false positives")]

pub mod attr_writer;
pub mod delimited;
pub mod dyn_attr;
pub mod err_collector;
pub mod extension;
pub mod last;
pub mod lazy;
pub mod lookahead_chain;
pub mod unpack;
