/// A VDF token type.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum TokenType {
    /// Left brace ('{').
    BraceLeft,
    /// Right brace ('}').
    BraceRight,
    /// A string.
    String,
}

/// A VDF token.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Token<'a> {
    /// The token [type](TokenType).
    pub r#type: TokenType,
    /// The slice of the token.
    pub lexeme: &'a [u8],
}

/// The scanner / lexer data.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Scanner<'a> {
    /// The source that is being scanned.
    pub source: &'a [u8],
    /// The start of the current lexeme.
    pub start: usize,
    /// The current position of the scanner.
    pub current: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new [`Scanner`].
    #[inline]
    pub const fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
        }
    }

    /// Checks if the scanner exhausted its [source](Scanner::source).
    #[inline]
    pub const fn is_finished(self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<u8> {
        let c = self.source.get(self.current).copied();
        self.current += 1;
        c
    }

    fn peek(self) -> Option<u8> {
        self.source.get(self.current).copied()
    }

    fn token(self, r#type: TokenType) -> Token<'a> {
        Token {
            r#type,
            lexeme: &self.source[self.start..self.current],
        }
    }

    fn string_tail(&mut self) -> Result<Token<'a>, Error> {
        loop {
            let next = self.peek();
            match next {
                Some(b'"') => {
                    self.current += 1;
                    break Ok(self.token(TokenType::String));
                }
                Some(b'\\') => self.current += 2,
                Some(_) => self.current += 1,
                None => break Err(Error::UnterminatedString),
            }
        }
    }
}

/// A [lexing](Scanner) error.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, thiserror::Error)]
pub enum Error {
    /// Unexpected token.
    #[error("unexpected token: '{}' ({0})", char::from(*.0))]
    UnexpectedToken(u8),
    /// Unterminated string literal.
    #[error("unterminated string")]
    UnterminatedString,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;
        let head = self.advance();
        // TODO: comments?
        match head {
            Some(c) if c.is_ascii_whitespace() => self.next(),
            Some(b'"') => Some(self.string_tail()),
            Some(b'{') => Some(Ok(self.token(TokenType::BraceLeft))),
            Some(b'}') => Some(Ok(self.token(TokenType::BraceRight))),
            Some(c) => Some(Err(Error::UnexpectedToken(c))),
            None => None,
        }
    }
}
