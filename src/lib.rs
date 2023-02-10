//! Switch Steam accounts.

#[cfg(not(target_os = "windows"))]
compile_error!("Only Windows is supported.");

mod username;
pub use username::{Username, UsernameError};

mod steam;
pub use steam::{Error, Result, Steam};

pub mod vdf;

mod util;
