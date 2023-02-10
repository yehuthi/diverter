//! [VDF](https://developer.valvesoftware.com/wiki/KeyValues) file processing.
//!
//! This is used to extract data from Steam's installation, such as [`LoginUser`]s.

mod scanner;
use std::fmt::Debug;

pub use scanner::{Error as ScanError, Scanner, Token, TokenType};

mod parser;
pub use parser::{parse, Error as ParseError, Id as ExprId, Value};

use crate::util::OkIter;

use self::parser::Document;

/// A login user record.
#[derive(Clone, Copy)]
pub struct LoginUser<'a> {
    /// The user's username.
    pub username: &'a [u8],
    /// The user's nickname.
    pub nickname: &'a [u8],
    /// Whether the user can be auto logged in.
    pub allow_auto_login: bool,
}

impl<'a> Debug for LoginUser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginUser")
            .field(
                "username",
                &format_args!("\"{}\"", self.username.escape_ascii()),
            )
            .field(
                "nickname",
                &format_args!("\"{}\"", self.nickname.escape_ascii()),
            )
            .field("allow_auto_login", &self.allow_auto_login)
            .finish()
    }
}

/// Error parsing a [`LoginUser`] from a loginusers.vdf file.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, thiserror::Error)]
pub enum LoginUserVdfError {
    /// Missing "users" key.
    #[error("missing expected \"users\" subkeys in loginusers.vdf")]
    ExpectedUsersSubkeys,
    /// Missing account name.
    #[error("missing expected \"AccountName\" key for user in loginusers.vdf")]
    ExpectedAccountNameKey,
    /// Missing persona name (nickname).
    #[error("missing expected \"PersonaName\" key for user in loginusers.vdf")]
    ExpectedPersonaNameKey,
    /// "users" key isn't associated with subkeys.
    #[error("expected \"users\" key (which was found) to have subkeys associated with it in loginusers.vdf")]
    ExpectedUserEntryToBeSubkeys,
}

impl<'a> LoginUser<'a> {
    /// Read [`LoginUser`]s from a VDF [`Document`].
    pub fn from_vdf(
        document: &'a Document,
    ) -> Result<
        impl Iterator<Item = Result<LoginUser<'a>, LoginUserVdfError>> + 'a,
        LoginUserVdfError,
    > {
        let users_sub = document
            .subkeys(ExprId::ROOT, b"users")
            .ok_or(LoginUserVdfError::ExpectedUsersSubkeys)?;
        let user_ids = document.0.iter().filter(move |row| row.parent == users_sub);
        Ok(user_ids.map(|user_sub| {
            if let Value::Subkeys(user_keyvals) = user_sub.value {
                Ok(Self {
                    username: document
                        .value_str(user_keyvals, b"AccountName")
                        .ok_or(LoginUserVdfError::ExpectedAccountNameKey)?,
                    nickname: document
                        .value_str(user_keyvals, b"PersonaName")
                        .ok_or(LoginUserVdfError::ExpectedPersonaNameKey)?,
                    allow_auto_login: document
                        .value_str(user_keyvals, b"AllowAutoLogin")
                        .map_or(false, |value| value != b"0"),
                })
            } else {
                Err(LoginUserVdfError::ExpectedUserEntryToBeSubkeys)
            }
        }))
    }
}

/// [Scan](ScanError) or [parse](ParseError) error.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, thiserror::Error)]
pub enum ScanParseError {
    /// Scan error.
    #[error("lexing error: {0}")]
    ScanError(#[from] ScanError),
    /// Parse error.
    #[error("parsing error: {0}")]
    ParseError(#[from] ParseError),
}

/// Scans and parses the source text.
pub fn scan_parse(source: &[u8]) -> Result<Document, ScanParseError> {
    let mut tokens = OkIter::new(Scanner::new(source));
    let result = parse(&mut tokens);
    match tokens.to_error() {
        Some(&e) => Err(e.into()),
        None => result.map_err(ScanParseError::ParseError),
    }
}
