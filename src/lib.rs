use std::io::{Read, Error, ErrorKind, Result};
use std::convert::From;
use std::str;
use std::fmt;
use std::error;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcfToken {
    String(String),
    DictStart,
    DictEnd,
}
#[derive(Debug)]
pub struct UnexpectedCharacter(pub char);
impl error::Error for UnexpectedCharacter {}
impl fmt::Display for UnexpectedCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected Character '{:?}'", self.0)
    }
}
impl<A> From<UnexpectedCharacter> for Result<A> {
    fn from(err: UnexpectedCharacter) -> Self {
        Err(Error::new(ErrorKind::InvalidData, err))
    }
}
#[derive(Debug)]
pub struct UnexpectedToken(pub AcfToken);
impl error::Error for UnexpectedToken {}
impl fmt::Display for UnexpectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected Token {:?}", self.0)
    }
}
impl<A> From<UnexpectedToken> for Result<A> {
    fn from(err: UnexpectedToken) -> Self {
        Err(Error::new(ErrorKind::InvalidData, err))
    }
}
#[derive(Debug)]
pub struct UnterminatedString;
impl error::Error for UnterminatedString {}
impl fmt::Display for UnterminatedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unterminated String literal")
    }
}
impl<A> From<UnterminatedString> for Result<A> {
    fn from(err: UnterminatedString) -> Self {
        Err(Error::new(ErrorKind::InvalidData, err))
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
            Some(c) => UnexpectedCharacter(c).into(),
            None => Ok(None)
        }
    }
    pub fn expect_next(&mut self) -> Result<AcfToken> {
        let t = self.try_next()?;
        t.ok_or(Error::new(ErrorKind::UnexpectedEof, ""))
    }
    pub fn expect(&mut self, token: AcfToken) -> Result<()> {
        let t = self.expect_next()?;
        if t == token {
            Ok(())
        } else {
            UnexpectedToken(t).into()
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
    pub fn select_path(
        &mut self,
        path: impl Iterator<Item = impl AsRef<str>>,
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
                    .map_err(|e| Error::new(ErrorKind::Other, e))
                    .map(str::to_owned)
                    .map(Some);
            }
            // TODO: handle escape sequences and utf8?
            Some(c) => buf.push(c as u8),
            None => { return UnterminatedString.into(); }
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
