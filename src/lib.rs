use std::io::{self, Read, ErrorKind, Result};
use std::convert::From;
use std::str;
use thiserror::Error;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcfToken {
    String(String),
    DictStart,
    DictEnd,
}
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Unexpected Character '{0:?}'")]
    UnexpectedCharacter(char),
    #[error("Unexpected Token {0:?}")]
    UnexpectedToken(AcfToken),
    #[error("Unterminated String literal")]
    UnterminatedString,
    #[error("Path not found {0:?}")]
    PathNotFound(Vec<String>),
}
impl From<TokenError> for io::Error {
    fn from(err: TokenError) -> Self {
        io::Error::new(ErrorKind::InvalidData, err)
    }
}

pub struct AcfTokenStream<R: Read>{
    read: R,
    depth: i64,
}
impl<R: Read> AcfTokenStream<R> {
    pub fn new(read: R) -> Self {
        Self {
            read,
            depth: 0,
        }
    }
    pub fn try_next(&mut self) -> Result<Option<AcfToken>> {
        match next_non_whitespace_char(&mut self.read)? {
            Some('{') => {
                self.depth += 1;
                Ok(Some(AcfToken::DictStart))
            }
            Some('}') => {
                self.depth -= 1;
                Ok(Some(AcfToken::DictEnd))
            }
            Some('"') => parse_str(&mut self.read)
                .map(|o| o.map(AcfToken::String)),
            Some(c) => Err(TokenError::UnexpectedCharacter(c).into()),
            None => Ok(None)
        }
    }
    pub fn expect_next(&mut self) -> Result<AcfToken> {
        let t = self.try_next()?;
        t.ok_or(io::Error::new(ErrorKind::UnexpectedEof, ""))
    }
    pub fn expect(&mut self, token: AcfToken) -> Result<()> {
        let t = self.expect_next()?;
        if t == token {
            Ok(())
        } else {
            Err(TokenError::UnexpectedToken(t).into())
        }
    }
    pub fn select(&mut self, target: impl AsRef<str>) -> Result<Option<()>> {
        while let Some(t) = self.try_next()? {
            match t {
                AcfToken::String(field) => if field == target.as_ref() {
                    return Ok(Some(()))
                }
                AcfToken::DictEnd => { break; }
                AcfToken::DictStart => { self.close_dict()?; }
            }
        }
        Ok(None)
    }
    pub fn try_select_path(
        &mut self,
        path: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Option<()>> {
        let mut not_first = false;
        for field in path {
            if not_first {
                self.expect(AcfToken::DictStart)?;
            } else {
                not_first = true;
            }
            match self.select(field)? {
                None => { return Ok(None) }
                Some(_) => {}
            }
        }
        Ok(Some(()))
    }
    pub fn select_path(
        &mut self,
        path: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<()> {
        let path_vec: Vec<String> = path.into_iter()
                .map(|s| s.as_ref().to_owned())
                .collect();
        let r = self.try_select_path(path_vec.clone())?;
        r.ok_or_else(|| io::Error::from(TokenError::PathNotFound(path_vec)))
    }
    pub fn close_dict(&mut self) -> Result<()> {
        self.skip_to_depth(self.depth - 1)
    }
    fn skip_to_depth(&mut self, target_depth: i64) -> Result<()> {
        while let Some(_) = self.try_next()? {
            if self.depth == target_depth {
                break;
            }
        }
        Ok(())
    }
}
fn next_char<R: Read>(mut reader: R) -> Result<Option<char>> {
    let mut buf: [u8; 1] = [0];
    let size = reader.read(&mut buf)?;
    Ok(if size == 1 {
        Some(buf[0] as char)
    } else {
        None
    })
}
fn next_non_whitespace_char<R: Read>(mut reader: R) -> Result<Option<char>> {
    loop {
        match next_char(&mut reader)? {
            Some(c) => if !c.is_whitespace() {
                return Ok(Some(c))
            },
            None => { return Ok(None) }
        }
    }
}
fn parse_str<R: Read>(mut reader: R) -> Result<Option<String>> {
    let mut buf: Vec<u8> = vec![];
    loop {
        match next_char(&mut reader)? {
            Some('"') => {
                return str::from_utf8(&buf)
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))
                    .map(str::to_owned)
                    .map(Some);
            }
            // TODO: handle escape sequences and utf8?
            Some(c) => buf.push(c as u8),
            None => { return Err(TokenError::UnterminatedString.into()); }
        }
    }
}
impl<R: Read> Iterator for AcfTokenStream<R> {
    type Item = Result<AcfToken>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(t)  => t.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
