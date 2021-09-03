use std::io::{self, ErrorKind, Read};

use thiserror::Error;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcfToken {
    String(String),
    DictStart,
    DictEnd,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected Character '{0:?}'")]
    UnexpectedCharacter(char),
    #[error("Unterminated String literal")]
    UnterminatedString,
}

fn format_path(items: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut s = String::new();
    for p in items {
        s.push('.');
        s.push_str(p.as_ref());
    }
    s
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Generic I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid data: {0}")]
    Parse(#[from] ParseError),
    #[error("Unexpected Token {0:?}")]
    UnexpectedToken(AcfToken),
    #[error("Path not found {}", format_path(&.0[..]))]
    PathNotFound(Vec<String>),
}

type Res<A> = Result<A, StreamError>;

pub struct AcfTokenStream<R> {
    read: R,
    depth: i64,
}
impl<R: Read> AcfTokenStream<R> {
    pub fn new(read: R) -> Self {
        Self { read, depth: 0 }
    }

    pub fn depth(&self) -> i64 {
        self.depth
    }

    pub fn try_next(&mut self) -> Res<Option<AcfToken>> {
        Ok(match next_non_whitespace_char(&mut self.read)? {
            Some('{') => {
                self.depth += 1;
                Some(AcfToken::DictStart)
            }
            Some('}') => {
                self.depth -= 1;
                Some(AcfToken::DictEnd)
            }
            Some('"') => {
                let s = parse_str(&mut self.read)?;
                s.map(AcfToken::String)
            }
            Some(c) => {
                Err(StreamError::from(ParseError::UnexpectedCharacter(c)))?
            },
            None => None,
        })
    }

    pub fn expect_next(&mut self) -> Res<AcfToken> {
        self.try_next()?
            .ok_or(StreamError::from(io::Error::new(ErrorKind::UnexpectedEof, "")))
    }

    pub fn expect(&mut self, token: AcfToken) -> Res<()> {
        let t = self.expect_next()?;
        if t == token {
            Ok(())
        } else {
            Err(StreamError::UnexpectedToken(t))
        }
    }

    pub fn select(&mut self, target: impl AsRef<str>) -> Res<Option<()>> {
        while let Some(t) = self.try_next()? {
            match t {
                AcfToken::String(field) => {
                    if field == target.as_ref() {
                        return Ok(Some(()));
                    }
                }
                AcfToken::DictEnd => {
                    break;
                }
                AcfToken::DictStart => {
                    self.close_dict()?;
                }
            }
        }
        Ok(None)
    }

    pub fn try_select_path(
        &mut self,
        path: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Res<Option<()>> {
        let mut not_first = false;
        for field in path {
            if not_first {
                self.expect(AcfToken::DictStart)?;
            } else {
                not_first = true;
            }
            if self.select(field)?.is_none() {
                return Ok(None);
            }
        }
        Ok(Some(()))
    }

    pub fn select_path<'a, T, U: AsRef<str>>(&mut self, path: &'a T) -> Res<()>
    where
        &'a T: IntoIterator<Item = U>,
    {
        self.try_select_path(path)?.ok_or_else(|| {
            let owned = path.into_iter().map(|s| s.as_ref().to_owned()).collect();
            StreamError::PathNotFound(owned)
        })
    }

    pub fn close_dict(&mut self) -> Res<()> {
        self.skip_to_depth(self.depth - 1)
    }

    fn skip_to_depth(&mut self, target_depth: i64) -> Res<()> {
        while self.depth > target_depth { 
            self.expect_next()?;
        }
        Ok(())
    }
}

// TODO: handle UTF-8 better, possibly by making this work with bytes and letting parse_str handle it
fn next_char<R: Read>(mut reader: R) -> io::Result<Option<char>> {
    let mut buf: [u8; 1] = [0];
    Ok(if reader.read(&mut buf)? == 1 {
        Some(buf[0] as char)
    } else {
        None
    })
}

fn next_non_whitespace_char<R: Read>(mut reader: R) -> io::Result<Option<char>> {
    while let Some(c) = next_char(&mut reader)? {
        if !c.is_whitespace() {
            return Ok(Some(c));
        }
    }
    Ok(None)
}

fn parse_str<R: Read>(mut reader: R) -> Res<Option<String>> {
    let mut buf = String::new();
    loop {
        match next_char(&mut reader)? {
            Some('"') => return Ok(Some(buf)),
            // TODO: handle escape sequences?
            Some(c) => buf.push(c),
            None => return Err(StreamError::from(ParseError::UnterminatedString)),
        }
    }
}

impl<R: Read> Iterator for AcfTokenStream<R> {
    type Item = Res<AcfToken>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}
