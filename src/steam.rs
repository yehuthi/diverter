//! Steam client operations.

use std::{
    ffi::c_char,
    fmt::Debug,
    fs::File,
    io,
    mem::MaybeUninit,
    os::windows::prelude::{FromRawHandle, OsStringExt, RawHandle},
    process::ExitCode,
};

use winapi::{
    ctypes::wchar_t,
    shared::minwindef::{DWORD, MAX_PATH},
};

use crate::{Username, UsernameError};

#[repr(C)]
#[derive(Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
/// A handle to the installed Steam client.
pub struct Steam {
    len: wchar_t,
    path: [wchar_t; MAX_PATH],
}

impl Debug for Steam {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Steam")
            .field("len", &self.len)
            .field("path", &std::ffi::OsString::from_wide(&self.path))
            .finish()
    }
}

/// Reflects `windows.c`'s `phase_t`.
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
    FileOpenVdf,
}

/// Reflects `windows.c`'s `result_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CResult {
    phase: CPhase,
    win_code: DWORD,
}

/// The primary error type.
#[derive(Debug, thiserror::Error)]
#[repr(u32)]
pub enum Error {
    /// Indicates failure to read registry in Steam's subkey.
    #[error("failed to read registry in Steam's subkey: {0}")]
    ReadSteamRegistry(io::Error) = 1,
    /// Indicates failure to set a registry value in Steam's subkey.
    #[error("failed to write registry in Steam's subkey: {0}")]
    WriteSteamRegistry(io::Error),
    /// Indicates failure to launch Steam.
    #[error("failed to launch Steam: {0}")]
    LaunchSteam(io::Error),
    /// Indicates failure while waiting for Steam to exit.
    #[error("failed to wait for Steam to exit: {0}")]
    WaitSteamExit(io::Error),
    /// Indicates failure to enumerate processes in-order to find Steam's processes.
    #[error("failed to search for a Steam process: {0}")]
    EnumProcesses(io::Error),
    /// Indicates failure to terimnate a Steam process.
    #[error("failed to terminate Steam's process: {0}")]
    KillSteam(io::Error),
    /// Indicates an invalid username was found in the Windows registry.
    #[error("the auto-login username in the registry is invalid: {0}")]
    InvalidUsernameInRegistry(UsernameError),
    /// Indicates failure to open a VDF file.
    #[error("failed to open a VDF file: {0}")]
    VdfOpen(io::Error),
}

/// Exit codes per `sysexits.h`.
impl<'a> From<&'a Error> for ExitCode {
    fn from(e: &'a Error) -> Self {
        ExitCode::from(match e {
            Error::InvalidUsernameInRegistry(_) => 78,
            _ => 69,
        })
    }
}

impl From<CResult> for Option<Error> {
    #[inline]
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
            CPhase::FileOpenVdf => Some(Error::VdfOpen(io::Error::from_raw_os_error(
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
    fn steam_is_running(steam: *const Steam, is_running: *mut u8) -> CResult;
    fn steam_vdf_loginusers(steam: *const Steam, file: *mut RawHandle) -> CResult;
}

/// Converts an error [`Option`] into a [`Result`](::std::result::Result).
///
/// - [`Some(error)`](Some) yield [`Err(error)`](Err).
/// - [`None`] yields [`Ok(value)`](Ok) where value is the given `value` argument.
fn err_opt<T, E>(error: Option<E>, value: T) -> ::std::result::Result<T, E> {
    if let Some(e) = error {
        Err(e)
    } else {
        Ok(value)
    }
}

/// A [`Steam`] [`Result`](::std::result::Result) type.
pub type Result<T> = ::std::result::Result<T, Error>;

impl Steam {
    /// Attempts to create a new [`Steam`] handle.
    #[inline]
    pub fn new() -> Result<Self> {
        let mut steam = Steam {
            len: 0,
            path: [0; MAX_PATH],
        };
        err_opt(unsafe { steam_init(&mut steam) }.into(), steam)
    }

    /// Gracefully shuts down Steam, if running.
    #[inline]
    pub fn shutdown(&self) -> Result<()> {
        err_opt(unsafe { steam_shutdown(self) }.into(), ())
    }

    /// Launches Steam.
    ///
    /// See also: [`Self::launch_fast`].
    #[inline]
    pub fn launch(&self) -> Result<()> {
        err_opt(unsafe { steam_launch(self) }.into(), ())
    }

    /// Launches Steam, skipping Steam's file checks.
    #[inline]
    pub fn launch_fast(&self) -> Result<()> {
        err_opt(unsafe { steam_launch_fast(self) }.into(), ())
    }

    /// Kills all Steam processes.
    ///
    /// Returns whether any were found and killed.
    #[inline]
    pub fn kill(&self) -> Result<bool> {
        let mut killed = 0u8;
        err_opt(unsafe { steam_kill(self, &mut killed) }.into(), killed != 0)
    }

    /// Sets the Steam user that Steam will attempt to automatically log into.
    #[inline]
    pub fn set_auto_login_user(username: Username) -> Result<()> {
        let username = username.as_bytes_with_nul();
        err_opt(
            unsafe { steam_set_auto_login_user(username.as_ptr() as *const i8, username.len()) }
                .into(),
            (),
        )
    }

    /// Gets the Steam user that Steam will attempt to automatically log into.
    #[inline]
    pub fn get_auto_login_user() -> Result<Username> {
        let mut data = [MaybeUninit::<u8>::uninit(); Username::MAX_LEN + 1];
        let mut len = data.len();
        err_opt(
            (unsafe { steam_get_auto_login_user(data.as_mut_ptr() as *mut i8, &mut len) }).into(),
            (),
        )?;
        let username = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, len - 1) };
        Username::try_from(username).map_err(Error::InvalidUsernameInRegistry)
    }

    /// Checks if the Steam client is running.
    #[inline]
    pub fn is_running(&self) -> Result<bool> {
        let mut is_running = 0;
        err_opt(
            unsafe { steam_is_running(self, &mut is_running) }.into(),
            is_running != 0,
        )
    }

    /// Gets a [file handle](File) to the `loginusers.vdf` file.
    #[inline]
    pub fn vdf_loginusers(&self) -> Result<File> {
        let mut handle: RawHandle = std::ptr::null_mut();
        err_opt(
            unsafe { steam_vdf_loginusers(self, &mut handle) }.into(),
            (),
        )?;
        Ok(unsafe { File::from_raw_handle(handle) })
    }
}
