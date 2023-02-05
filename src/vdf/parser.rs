use super::Token;

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyValue<'a> {
    pub parent: usize,
    pub key: &'a [u8],
    pub value: &'a [u8],
}

pub type Document<'a> = Vec<KeyValue<'a>>;

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
    parent: usize,
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
                    document.push(KeyValue {
                        parent,
                        key: unsurround(name.lexeme),
                        value: unsurround(value.lexeme),
                    });
                    Ok(ParseOneTerminal::Yield)
                }
                super::TokenType::BraceLeft => {
                    let parent = name.lexeme.as_ptr() as usize;
                    loop {
                        let piece = parse_one(tokens, document, parent, true)?;
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
    let mut document = Document::new();
    loop {
        if parse_one(&mut tokens, &mut document, !0, false)? != ParseOneTerminal::Eof {
            break;
        }
    }
    Ok(document)
}
