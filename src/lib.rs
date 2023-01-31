//! Switch Steam accounts.

#[cfg(not(target_os = "windows"))]
compile_error!("Only Windows is supported.");

mod windows;
pub use windows::{get_auto_login_user, set_auto_login_user};

mod username;
pub use username::{Username, UsernameError};

mod steam;
pub use steam::Steam;
