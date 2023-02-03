use std::{ffi::c_char, fmt::Debug, io, mem::MaybeUninit, os::windows::prelude::OsStringExt};

use winapi::{
    ctypes::wchar_t,
    shared::minwindef::{DWORD, MAX_PATH},
};

use crate::Username;

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
    ReadSteamRegistry,
    WriteSteamRegistry,
    CanonicalizeSteamPath,
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

impl From<CResult> for io::Error {
    fn from(result: CResult) -> Self {
        Self::from_raw_os_error(result.win_code as i32)
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

#[derive(Debug)]
pub enum ShutdownError {
    LaunchSteamError(io::Error),
    WaitSteamError(io::Error),
}

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

    pub fn shutdown(&self) -> Result<(), ShutdownError> {
        let result = unsafe { steam_shutdown(self) };
        match result.phase {
            CPhase::Ok => Ok(()),
            CPhase::LaunchSteam => Err(ShutdownError::LaunchSteamError(result.into())),
            CPhase::WaitSteamExit => Err(ShutdownError::WaitSteamError(result.into())),
            _ => unreachable!(),
        }
    }

    pub fn launch(&self) -> io::Result<()> {
        let result = unsafe { steam_launch(self) };
        if result.phase == CPhase::Ok {
            Ok(())
        } else {
            Err(result.into())
        }
    }

    pub fn launch_fast(&self) -> io::Result<()> {
        let result = unsafe { steam_launch_fast(self) };
        if result.phase == CPhase::Ok {
            Ok(())
        } else {
            Err(result.into())
        }
    }

    pub fn kill(&self) -> io::Result<bool> {
        let mut killed = 0u8;
        let result = unsafe { steam_kill(self, &mut killed) };
        match result.phase {
            CPhase::Ok => Ok(killed != 0),
            _ => Err(io::Error::from_raw_os_error(result.win_code as _)),
        }
    }

    pub fn set_auto_login_user(username: Username) -> io::Result<()> {
        let username = username.as_bytes_with_nul();
        let result =
            unsafe { steam_set_auto_login_user(username.as_ptr() as *const i8, username.len()) };
        if result.phase == CPhase::Ok {
            Ok(())
        } else {
            Err(result.into())
        }
    }

    pub fn get_auto_login_user() -> io::Result<Username> {
        let mut data = [MaybeUninit::uninit(); Username::MAX_LEN + 1];
        let mut len = data.len();
        let result = unsafe { steam_get_auto_login_user(data.as_mut_ptr() as *mut i8, &mut len) };
        if result.phase == CPhase::Ok {
            unsafe { Ok(Username::from_raw_parts(data, len)) }
        } else {
            Err(result.into())
        }
    }
}
