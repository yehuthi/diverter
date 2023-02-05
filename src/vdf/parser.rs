use super::Token;

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct Id(pub usize);

impl Id {
    pub const ROOT: Id = Id(!0);
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Value<'a> {
    String(&'a [u8]),
    Subkeys(Id),
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyValue<'a> {
    pub parent: Id,
    pub key: &'a [u8],
    pub value: Value<'a>,
}

#[derive(Debug, Hash, Default, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct Document<'a>(pub Vec<KeyValue<'a>>);

impl<'a> Document<'a> {
    pub fn subkeys(&self, at: Id, key: &'a [u8]) -> Option<Id> {
        let result = self.0.iter().find(|row| row.parent == at && row.key == key);
        match result {
            Some(KeyValue {
                value: Value::Subkeys(sub),
                ..
            }) => Some(*sub),
            _ => None,
        }
    }

    pub fn value_str(&self, at: Id, name: &[u8]) -> Option<&'a [u8]> {
        let result = self
            .0
            .iter()
            .find(|row| row.parent == at && row.key == name);
        match result {
            Some(KeyValue {
                value: Value::String(sub),
                ..
            }) => Some(*sub),
            _ => None,
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Error {
    UnexpectedBraceLeftNoName,
    UnexpectedBraceRightNoMatch,
    ExpectedKeyValueAfterKeyName,
}

fn unsurround(s: &[u8]) -> &[u8] {
    &s[1..s.len() - 1]
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
enum ParseOneTerminal {
    BlockEnd,
    Eof,
    Yield,
}

fn parse_one<'a>(
    tokens: &mut impl Iterator<Item = Token<'a>>,
    document: &mut Document<'a>,
    parent: Id,
    brace_terminal: bool,
) -> Result<ParseOneTerminal, Error> {
    let Some(head) = tokens.next() else { return Ok(ParseOneTerminal::Eof) };
    match head.r#type {
        super::TokenType::BraceLeft => Err(Error::UnexpectedBraceLeftNoName),
        super::TokenType::BraceRight => {
            if brace_terminal {
                Ok(ParseOneTerminal::BlockEnd)
            } else {
                Err(Error::UnexpectedBraceRightNoMatch)
            }
        }
        super::TokenType::String => {
            let name = head;
            let Some(value ) = tokens.next() else { return Err(Error::ExpectedKeyValueAfterKeyName) };
            match value.r#type {
                super::TokenType::String => {
                    document.0.push(KeyValue {
                        parent,
                        key: unsurround(name.lexeme),
                        value: Value::String(unsurround(value.lexeme)),
                    });
                    Ok(ParseOneTerminal::Yield)
                }
                super::TokenType::BraceLeft => {
                    let sub_parent = Id(name.lexeme.as_ptr() as usize);
                    document.0.push(KeyValue {
                        parent,
                        key: unsurround(name.lexeme),
                        value: Value::Subkeys(sub_parent),
                    });
                    loop {
                        let piece = parse_one(tokens, document, sub_parent, true)?;
                        if piece == ParseOneTerminal::BlockEnd {
                            break Ok(ParseOneTerminal::Yield);
                        }
                    }
                }
                super::TokenType::BraceRight => Err(Error::UnexpectedBraceRightNoMatch),
            }
        }
    }
}

pub fn parse<'a>(mut tokens: impl Iterator<Item = Token<'a>>) -> Result<Document<'a>, Error> {
    let mut document = Document::default();
    loop {
        if parse_one(&mut tokens, &mut document, Id::ROOT, false)? != ParseOneTerminal::Eof {
            break;
        }
    }
    Ok(document)
}
