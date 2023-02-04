#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum TokenType {
    BraceLeft,
    BraceRight,
    String,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Token {
    pub r#type: TokenType,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Scanner<'a> {
    pub source: &'a [u8],
    pub start: usize,
    pub current: usize,
}

impl<'a> Scanner<'a> {
    #[inline]
    pub const fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
        }
    }

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

    fn token(self, r#type: TokenType) -> Token {
        Token {
            r#type,
            start: self.start,
            end: self.current,
        }
    }

    fn string_tail(&mut self) -> Result<Token, Error> {
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

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Error {
    UnexpectedToken(u8),
    UnterminatedString,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, Error>;

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
