use std::{ffi::c_void, io};

use winapi::{shared::winerror, um::winreg};

use crate::Username;

const STEAM_SUBKEY: *const i8 = b"SOFTWARE\\Valve\\Steam\0" as *const u8 as *const i8;
const AUTO_LOGIN_USER_VALUE_NAME: *const i8 = b"AutoLoginUser\0" as *const u8 as *const i8;

/// Sets user of the given NUL-terminated username to be the user that the Steam client will attempt to automatically log-in to.
pub fn set_auto_login_user(username: &[u8]) -> io::Result<()> {
    use winapi::um::winnt::REG_SZ;

    let result = unsafe {
        winreg::RegSetKeyValueA(
            winreg::HKEY_CURRENT_USER,
            STEAM_SUBKEY,
            AUTO_LOGIN_USER_VALUE_NAME,
            REG_SZ,
            username.as_ptr() as _,
            username.len() as _,
        )
    };
    if result == winerror::ERROR_SUCCESS as _ {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(result))
    }
}

/// Gets the user that the Steam client will attempt to automatically log-in to, if exists.
pub fn get_auto_login_user(username: &mut [u8; Username::MAX_LEN + 1]) -> io::Result<usize> {
    let mut size = username.len() as _;
    let result = unsafe {
        winreg::RegGetValueA(
            winreg::HKEY_CURRENT_USER,
            STEAM_SUBKEY,
            AUTO_LOGIN_USER_VALUE_NAME,
            winreg::RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            username.as_ptr() as *mut c_void,
            &mut size,
        )
    };
    if result == winerror::ERROR_SUCCESS as _ {
        Ok(size as usize)
    } else {
        Err(io::Error::from_raw_os_error(result))
    }
}
