#[cfg(not(target_os = "windows"))]
compile_error!("Only Windows is supported.");

mod windows;

use std::{
    fmt::{Debug, Display},
    mem::MaybeUninit,
    str::FromStr,
};

pub use windows::set_auto_login_user;

#[derive(Clone, Copy)]
pub struct Username {
    data: [MaybeUninit<u8>; Username::MAX_LEN + /* null terminator */ 1],
    len: usize,
}

impl Debug for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Username")
            .field(
                "data",
                &format_args!("\"{}\"", self.as_bytes_with_nul().escape_ascii()),
            )
            .field("len", &self.len)
            .finish()
    }
}

impl Display for Username {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }, f)
    }
}

impl Username {
    pub const MAX_LEN: usize = 32;
    pub const MIN_LEN: usize = 3;

    #[inline(always)]
    pub const fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len) }
    }

    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len - 1) }
    }
}

impl<'a> TryFrom<&'a [u8]> for Username {
    type Error = UsernameError;

    #[inline]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() > Username::MAX_LEN {
            Err(UsernameError::TooLong)?;
        }
        if value.len() < Username::MIN_LEN {
            Err(UsernameError::TooShort)?;
        }
        let mut data = [MaybeUninit::uninit(); Username::MAX_LEN + 1];
        for (&src, dst) in value.iter().zip(data.iter_mut()) {
            if !matches!(src, b'a'..=b'z' | b'0'..=b'9' | b'A'..=b'Z' | b'_') {
                Err(UsernameError::IllegalCharacters)?
            }
            *dst = MaybeUninit::new(src.to_ascii_lowercase());
        }
        data[value.len()] = MaybeUninit::new(b'\0');

        Ok(Self {
            data,
            len: value.len() + 1,
        })
    }
}

impl<'a> TryFrom<&'a str> for Username {
    type Error = UsernameError;

    #[inline(always)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Username::try_from(value.as_bytes())
    }
}

impl FromStr for Username {
    type Err = UsernameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Username::try_from(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UsernameError {
    #[error("the username is too short")]
    TooShort,
    #[error("the username is too long")]
    TooLong,
    #[error("the username contains illegal characters, it must be ASCII")]
    IllegalCharacters,
}
