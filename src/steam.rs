use std::{ffi::c_char, fmt::Debug, io, mem::MaybeUninit, os::windows::prelude::OsStringExt};

use winapi::{
    ctypes::wchar_t,
    shared::minwindef::{DWORD, MAX_PATH},
};

use crate::{Username, UsernameError};

#[repr(C)]
#[derive(Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Steam {
    len: wchar_t,
    path: [wchar_t; MAX_PATH],
}

impl Debug for Steam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Steam")
            .field("len", &self.len)
            .field("path", &std::ffi::OsString::from_wide(&self.path))
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
enum CPhase {
    Ok = 0,
    ReadSteamRegistry = 1,
    WriteSteamRegistry,
    LaunchSteam,
    WaitSteamExit,
    EnumProcesses,
    KillSteam,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CResult {
    phase: CPhase,
    win_code: DWORD,
}

#[derive(Debug, thiserror::Error)]
#[repr(u32)]
pub enum Error {
    #[error("failed to read registry in Steam's subkey: {0}")]
    ReadSteamRegistry(io::Error) = 1,
    #[error("failed to write registry in Steam's subkey: {0}")]
    WriteSteamRegistry(io::Error),
    #[error("failed to launch Steam: {0}")]
    LaunchSteam(io::Error),
    #[error("failed to wait for Steam to exit: {0}")]
    WaitSteamExit(io::Error),
    #[error("failed to search for a Steam process: {0}")]
    EnumProcesses(io::Error),
    #[error("failed to terminate Steam's process: {0}")]
    KillSteam(io::Error),
    #[error("the auto-login username in the registry is invalid: {0}")]
    InvalidUsernameInRegistry(UsernameError),
}

impl From<CResult> for Option<Error> {
    fn from(value: CResult) -> Self {
        match value.phase {
            CPhase::Ok => None,
            CPhase::ReadSteamRegistry => Some(Error::ReadSteamRegistry(
                io::Error::from_raw_os_error(value.win_code as _),
            )),
            CPhase::WriteSteamRegistry => Some(Error::WriteSteamRegistry(
                io::Error::from_raw_os_error(value.win_code as _),
            )),
            CPhase::LaunchSteam => Some(Error::LaunchSteam(io::Error::from_raw_os_error(
                value.win_code as _,
            ))),
            CPhase::WaitSteamExit => Some(Error::WaitSteamExit(io::Error::from_raw_os_error(
                value.win_code as _,
            ))),
            CPhase::EnumProcesses => Some(Error::EnumProcesses(io::Error::from_raw_os_error(
                value.win_code as _,
            ))),
            CPhase::KillSteam => Some(Error::KillSteam(io::Error::from_raw_os_error(
                value.win_code as _,
            ))),
        }
    }
}

#[link(name = "windowsutil")]
extern "C" {
    fn steam_init(steam: *mut Steam) -> CResult;
    fn steam_shutdown(steam: *const Steam) -> CResult;
    fn steam_launch(steam: *const Steam) -> CResult;
    fn steam_launch_fast(steam: *const Steam) -> CResult;
    fn steam_kill(steam: *const Steam, killed: *mut u8) -> CResult;
    fn steam_set_auto_login_user(username: *const c_char, username_len: usize) -> CResult;
    fn steam_get_auto_login_user(username: *mut c_char, username_len: *mut usize) -> CResult;
}

fn err_opt<T, E>(error: Option<E>, value: T) -> ::std::result::Result<T, E> {
    if let Some(e) = error {
        Err(e)
    } else {
        Ok(value)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Steam {
    pub fn new() -> io::Result<Self> {
        let mut steam = Steam {
            len: 0,
            path: [0; MAX_PATH],
        };
        let result = unsafe { steam_init(&mut steam) };
        if result.phase == CPhase::Ok {
            Ok(steam)
        } else {
            Err(io::Error::from_raw_os_error(result.win_code as i32))
        }
    }

    pub fn shutdown(&self) -> Result<()> {
        err_opt(unsafe { steam_shutdown(self) }.into(), ())
    }

    pub fn launch(&self) -> Result<()> {
        err_opt(unsafe { steam_launch(self) }.into(), ())
    }

    pub fn launch_fast(&self) -> Result<()> {
        err_opt(unsafe { steam_launch_fast(self) }.into(), ())
    }

    pub fn kill(&self) -> Result<bool> {
        let mut killed = 0u8;
        err_opt(unsafe { steam_kill(self, &mut killed) }.into(), killed != 0)
    }

    pub fn set_auto_login_user(username: Username) -> Result<()> {
        let username = username.as_bytes_with_nul();
        err_opt(
            unsafe { steam_set_auto_login_user(username.as_ptr() as *const i8, username.len()) }
                .into(),
            (),
        )
    }

    pub fn get_auto_login_user() -> Result<Username> {
        let mut data = [MaybeUninit::uninit(); Username::MAX_LEN + 1];
        let mut len = data.len();
        err_opt(
            (unsafe { steam_get_auto_login_user(data.as_mut_ptr() as *mut i8, &mut len) }).into(),
            (),
        )?;
        // TODO: error-handle, this can violate invariants
        Ok(unsafe { Username::from_raw_parts(data, len) })
    }
}
