//! Switch Steam accounts.

#[cfg(not(target_os = "windows"))]
compile_error!("Only Windows is supported.");

mod windows;
pub use windows::set_auto_login_user;

mod username;
pub use username::{Username, UsernameError};
