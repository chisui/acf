use std::io::{self, Read};

use thiserror::Error;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcfToken {
    String(String),
    DictStart,
    DictEnd,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Generic I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Unexpected Character '{0:?}'")]
    UnexpectedCharacter(char),
    #[error("Unterminated String literal")]
    UnterminatedString,
    #[error("Unexpected EOF")]
    UnexpectedEof,
}

type Res<A> = Result<A, ParseError>;

pub struct AcfTokenStream<R> {
    read: R,
}
impl<R: Read> Iterator for AcfTokenStream<R> {
    type Item = Res<AcfToken>;
    fn next(&mut self) -> Option<Res<AcfToken>> {
        self.try_next().transpose()
    }
}
impl<R: Read> AcfTokenStream<R> {
    pub fn new(read: R) -> Self {
        Self { read }
    }

    pub fn try_next(&mut self) -> Res<Option<AcfToken>> {
        Ok(match self.next_non_whitespace_char()? {
            Some('{') => Some(AcfToken::DictStart),
            Some('}') => Some(AcfToken::DictEnd),
            Some('"') => self.parse_str()?,
            Some(c) => {
                Err(ParseError::UnexpectedCharacter(c))?
            },
            None => None,
        })
    }

    // TODO: handle UTF-8 better, possibly by making this work with bytes and letting parse_str handle it
    fn next_char(&mut self) -> io::Result<Option<char>> {
        let mut buf: [u8; 1] = [0];
        Ok(if self.read.read(&mut buf)? == 1 {
            Some(buf[0] as char)
        } else {
            None
        })
    }

    fn next_non_whitespace_char(&mut self) -> io::Result<Option<char>> {
        while let Some(c) = self.next_char()? {
            if !c.is_whitespace() {
                return Ok(Some(c));
            }
        }
        Ok(None)
    }

    fn parse_str(&mut self) -> Res<Option<AcfToken>> {
        let mut buf = String::new();
        loop {
            match self.next_char()? {
                Some('"') => return Ok(Some(AcfToken::String(buf))),
                // TODO: handle escape sequences?
                Some(c) => buf.push(c),
                None => return Err(ParseError::UnterminatedString),
            }
        }
    }
}
