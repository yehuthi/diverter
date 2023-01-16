use std::{ffi::CString, io};

mod sys {
    use std::ffi::{c_char, c_long, c_ulong, c_void};

    pub enum KEY {}
    pub type HKEY = *const KEY;
    pub const HKEY_CURRENT_USER: HKEY = 0x80000001usize as HKEY;
    pub type LPCSTR = *const c_char;
    pub type DWORD = c_ulong;
    pub const REG_SZ: DWORD = 1;
    pub const ERROR_SUCCESS: DWORD = 0;
    pub type LPCVOID = *const c_void;
    pub type LSTATUS = c_long;

    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegSetKeyValueA(
            key: HKEY,
            subKey: LPCSTR,
            valueName: LPCSTR,
            r#type: DWORD,
            data: LPCVOID,
            cbData: DWORD,
        ) -> LSTATUS;
    }
}

pub fn set_auto_login_user(username: &CString) -> io::Result<()> {
    const SUBKEY: sys::LPCSTR = b"SOFTWARE\\Valve\\Steam\0" as *const u8 as *const i8;
    const VALUE_NAME: sys::LPCSTR = b"AutoLoginUser\0" as *const u8 as *const i8;
    let username = username.to_bytes_with_nul();
    let result = unsafe {
        sys::RegSetKeyValueA(
            sys::HKEY_CURRENT_USER,
            SUBKEY,
            VALUE_NAME,
            sys::REG_SZ,
            username.as_ptr() as sys::LPCVOID,
            username.len() as sys::DWORD,
        )
    };
    if result == sys::ERROR_SUCCESS as sys::LSTATUS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(result))
    }
}
