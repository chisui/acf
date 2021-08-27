use std::io::{Read, Error, ErrorKind, Result};
use std::str;
use std::fmt;
use std::error;


#[derive(Clone, Debug)]
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
#[derive(Debug)]
pub struct UnexpectedToken(pub AcfToken);
impl error::Error for UnexpectedToken {}
impl fmt::Display for UnexpectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected Token {:?}", self.0)
    }
}

pub struct AcfTokenReader<R: Read>(pub R);
fn next_char<R: Read>(mut reader: R) -> Result<char> {
    let mut buf: [u8; 1] = [0];
    reader.read_exact(&mut buf)?;
    Ok(buf[0] as char)
}
fn next_non_whitespace_char<R: Read>(mut reader: R) -> Result<char> {
    let mut c = next_char(&mut reader)?;
    while c.is_whitespace() {
        c = next_char(&mut reader)?;
    }
    Ok(c)
}
fn parse_str<R: Read>(mut reader: R) -> Result<String> {
    let mut buf: Vec<u8> = vec![];
    loop {
        match next_char(&mut reader)? {
            '"' => {
                return str::from_utf8(&buf)
                    .map_err(|e| Error::new(ErrorKind::Other, e))
                    .map(str::to_owned);
            }
            // TODO: handle escape sequences and utf8?
            c => buf.push(c as u8),
        }
    }
}
impl<R: Read> Iterator for AcfTokenReader<R> {
    type Item = Result<AcfToken>;
    fn next(&mut self) -> Option<Result<AcfToken>> {
        next_non_whitespace_char(&mut self.0)
            .ok()
            .map(|t| {
                match t {
                    '{' => Ok(AcfToken::DictStart),
                    '}' => Ok(AcfToken::DictEnd),
                    '"' => parse_str(&mut self.0).map(AcfToken::String),
                    c => Err(Error::new(ErrorKind::Other, UnexpectedCharacter(c))),
                }
            })
    }
}
