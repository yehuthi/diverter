//! Steam [`Username`]s.

use std::{
    fmt::{self, Debug, Display, Formatter},
    mem::MaybeUninit,
    str::FromStr,
};

mod core {
    //! Core functionality for [`Username`].
    //!
    //! This is in its own module to reduce surface area for invariant invalidation.

    use super::*;

    /// A Steam username.
    ///
    /// # Validation
    /// A username must:
    /// - be at least [`Username::MIN_LEN`] (3) characters.
    /// - be at most [`Username::MAX_LEN`] (32) characters.
    /// - only consist of characters matching the class `[a-zA-Z0-9_]`.
    #[derive(Clone, Copy)]
    pub struct Username {
        /// The username in lowercase with a NUL-terminator.
        data: [MaybeUninit<u8>; Username::MAX_LEN + /* null terminator */ 1],
        /// The length of the username, excluding NUL-terminator.
        len: usize,
    }

    impl Username {
        /// Gets the username's length.
        #[inline(always)]
        #[allow(clippy::len_without_is_empty)] // usernames can't be empty
        pub fn len(&self) -> usize {
            self.len
        }

        /// Gets a slice of the username as ASCII bytes with a NUL character terminator (see also [`Username::as_bytes`]).
        #[inline(always)]
        pub const fn as_bytes_with_nul(&self) -> &[u8] {
            // SAFETY:
            // per field invariants
            unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len) }
        }
    }

    impl<'a> TryFrom<&'a [u8]> for Username {
        type Error = UsernameError;

        #[inline]
        fn try_from(username: &'a [u8]) -> Result<Self, Self::Error> {
            if username.len() >= Username::MAX_LEN {
                Err(UsernameError::TooLong)?;
            }
            if username.len() <= Username::MIN_LEN {
                Err(UsernameError::TooShort)?;
            }
            let mut data = [MaybeUninit::uninit(); Username::MAX_LEN + 1];
            for (&src, dst) in username.iter().zip(data.iter_mut()) {
                if !matches!(src, b'a'..=b'z' | b'0'..=b'9' | b'A'..=b'Z' | b'_') {
                    Err(UsernameError::IllegalCharacters)?
                }
                *dst = MaybeUninit::new(src.to_ascii_lowercase());
            }

            data[username.len()] = MaybeUninit::new(b'\0');

            Ok(Self {
                data,
                len: username.len() + /* NUL terminator */ 1,
            })
        }
    }
}

pub use self::core::*;

impl Debug for Username {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Username")
            .field(
                "data",
                &format_args!("\"{}\"", self.as_bytes_with_nul().escape_ascii()),
            )
            .field("len", &self.len())
            .finish()
    }
}

impl Username {
    /// The maximum length of a [`Username`].
    pub const MAX_LEN: usize = 32;
    /// The minimum length of a [`Username`].
    pub const MIN_LEN: usize = 3;

    /// Gets a slice of the username as ASCII bytes (see also [`Username::as_bytes_with_nul`]).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        let with_nul = self.as_bytes_with_nul();
        &with_nul[..with_nul.len() - 1]
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

impl AsRef<str> for Username {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        // SAFETY: field invariants guarantee a subset of ASCII
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl Display for Username {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(AsRef::<str>::as_ref(self), f)
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
    #[error("the username contains illegal characters, only ASCII alphanumeric characters and underscore (_) are allowed")]
    IllegalCharacters,
}
