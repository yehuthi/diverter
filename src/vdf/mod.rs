mod scanner;
use std::fmt::Debug;

pub use scanner::{Error as ScanError, Scanner, Token, TokenType};

mod parser;
pub use parser::{parse, Error as ParseError, Id as ExprId, Value};

use self::parser::Document;

#[derive(Clone, Copy)]
pub struct LoginUser<'a> {
    pub username: &'a [u8],
    pub nickname: &'a [u8],
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

impl<'a> LoginUser<'a> {
    pub fn from_vdf(document: &'a Document) -> impl Iterator<Item = LoginUser<'a>> + 'a {
        let users_sub = document.subkeys(ExprId::ROOT, b"users").unwrap();
        let user_ids = document
            .0
            .iter()
            .filter(move |row| row.parent == users_sub)
            .map(|user_sub| {
                if let Value::Subkeys(value) = user_sub.value {
                    value
                } else {
                    panic!()
                }
            });
        user_ids.map(|user_sub| Self {
            username: document.value_str(user_sub, b"AccountName").unwrap(),
            nickname: document.value_str(user_sub, b"PersonaName").unwrap(),
            allow_auto_login: document.value_str(user_sub, b"AllowAutoLogin").unwrap() != b"0",
        })
    }
}
