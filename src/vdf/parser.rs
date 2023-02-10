use super::Token;

/// A [`Document`] element ID.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct Id(pub usize);

impl Id {
    /// The [`Id`] for the root of the document.
    pub const ROOT: Id = Id(!0);
}

/// A key value.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Value<'a> {
    /// A string value.
    String(&'a [u8]),
    /// Subkeys value.
    Subkeys(Id),
}

/// A key-value pair.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyValue<'a> {
    /// Where the key-value is specified.
    pub parent: Id,
    /// The key part.
    pub key: &'a [u8],
    /// The value part.
    pub value: Value<'a>,
}

/// A VDF document.
#[derive(Debug, Hash, Default, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct Document<'a>(pub Vec<KeyValue<'a>>);

impl<'a> Document<'a> {
    /// Gets the subkeys at the given path.
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

    /// Gets the value at the given path.
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

/// Parse error.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, thiserror::Error)]
pub enum Error {
    /// Unexpected left brace which indicates subkeys but there's no preceding key name for them.
    #[error("unexpected left brace ('{{'), there's no preceding key name to specify subkeys")]
    UnexpectedBraceLeftNoName,
    /// Unexpected / unmatching right brace.
    #[error("unexpected right brace ('}}'), there's no matching left brace.")]
    UnexpectedBraceRightNoMatch,
    /// Unexpected EOF after key name.
    #[error("expected key value after key name but reached EOF")]
    ExpectedKeyValueAfterKeyName,
}

/// Removes the first and last characters.
///
/// Useful to remove surrounding characters like quotes.
fn unsurround(s: &[u8]) -> &[u8] {
    &s[1..s.len() - 1]
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
enum ParseOneTerminal {
    BlockEnd,
    Eof,
    Yield,
}

/// Parses a single element.
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

/// Parses a [`Document`].
pub fn parse<'a>(mut tokens: impl Iterator<Item = Token<'a>>) -> Result<Document<'a>, Error> {
    let mut document = Document::default();
    loop {
        if parse_one(&mut tokens, &mut document, Id::ROOT, false)? != ParseOneTerminal::Eof {
            break;
        }
    }
    Ok(document)
}
