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
impl<R: Read> AcfTokenReader<R> {
    pub fn skip_to_field(&mut self, target: &str) -> Result<Option<AcfToken>> {
        while let Some(t) = self.next() {
            match t? {
                AcfToken::String(field) => if field == target {
                    return self.expect_next().map(Some);
                }
                AcfToken::DictEnd => { break; }
                _ => {}
            }
            self.skip_field_value(0)?;
        }
        Ok(None)
    }
    pub fn expect_next(&mut self) -> Result<AcfToken> {
        let t = next_non_whitespace_char(&mut self.0)?;
        match t {
            '{' => Ok(AcfToken::DictStart),
            '}' => Ok(AcfToken::DictEnd),
            '"' => parse_str(&mut self.0).map(AcfToken::String),
            c => Err(Error::new(ErrorKind::Other, UnexpectedCharacter(c))),
        }
    }
    pub fn skip_field_value(&mut self, current_depth: i64) -> Result<()> {
        let mut depth = current_depth;
        while let Some(t) = self.next() {
            match t? {
                AcfToken::String(_) => {}
                AcfToken::DictStart => { depth += 1; }
                AcfToken::DictEnd   => { depth -= 1; }
            }
            if depth == 0 {
                break;
            }
        }
        Ok(())
    }
}
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
