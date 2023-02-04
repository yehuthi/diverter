//! Steam [`Username`]s.

use std::{
    fmt::{self, Debug, Display, Formatter},
    mem::MaybeUninit,
    str::FromStr,
};

/// A Steam username.
///
/// # Validation
/// A username must:
/// - be at least [`Username::MIN_LEN`] (3) characters.
/// - be at most [`Username::MAX_LEN`] (32) characters.
/// - only consist of characters matching the class `[a-zA-Z0-9_]`.
#[derive(Clone, Copy)]
pub struct Username {
    data: [MaybeUninit<u8>; Username::MAX_LEN + /* null terminator */ 1],
    len: usize,
}

impl Debug for Username {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }, f)
    }
}

impl Username {
    /// The maximum length of a [`Username`].
    pub const MAX_LEN: usize = 32;
    /// The minimum length of a [`Username`].
    pub const MIN_LEN: usize = 3;

    /// Gets a slice of the username as ASCII bytes with a NUL character terminator (see also [`Username::as_bytes`]).
    #[inline(always)]
    pub const fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len) }
    }

    /// Gets a slice of the username as ASCII bytes (see also [`Username::as_bytes_with_nul`]).
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

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Username::try_from(s)
    }
}

/// A [`Username`] validation error.
///
/// See [`Username`]'s validation doc section for specifics.
#[derive(Debug, thiserror::Error)]
pub enum UsernameError {
    /// The username is too short.
    #[error(
        "the username is too short, it must be at least {} characters",
        Username::MIN_LEN
    )]
    TooShort,
    /// The username is too long.
    #[error(
        "the username is too long, it must be at most {} characters",
        Username::MAX_LEN
    )]
    TooLong,
    /// The username is contains illegal characters.
    #[error("the username contains illegal characters, it must be ASCII")]
    IllegalCharacters,
}
