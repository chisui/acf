use std::io::Read;

use thiserror::Error;


pub use crate::parse::{
    AcfToken,
    AcfTokenStream,
    ParseError,
};

pub mod parse;


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
    #[error("Invalid data: {0}")]
    Parse(#[from] ParseError),
    #[error("Unexpected Token {0:?}")]
    UnexpectedToken(AcfToken),
    #[error("Path not found {}", format_path(&.0[..]))]
    PathNotFound(Vec<String>),
}

type Res<A = ()> = Result<A, StreamError>;

pub struct StructuredAcfTokenStream<R> {
    read: AcfTokenStream<R>,
    depth: i64,
}
impl<R: Read> Iterator for StructuredAcfTokenStream<R> {
    type Item = Res<AcfToken>;
    fn next(&mut self) -> Option<Res<AcfToken>> {
        self.try_next().transpose()
    }
}
impl<R: Read> StructuredAcfTokenStream<R> {
    pub fn new(read: AcfTokenStream<R>) -> Self {
        Self { read, depth: 0 }
    }

    pub fn depth(&self) -> i64 {
        self.depth
    }

    pub fn try_next(&mut self) -> Res<Option<AcfToken>> {
        let t = self.read.try_next()?;
        self.depth += match t {
            Some(AcfToken::DictStart) => 1,
            Some(AcfToken::DictEnd) => -1,
            _ => 0,
        };
        Ok(t)
    }

    pub fn expect_next(&mut self) -> Res<AcfToken> {
        self.try_next()?
            .ok_or(StreamError::from(ParseError::UnexpectedEof))
    }

    pub fn expect(&mut self, token: AcfToken) -> Res {
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

    pub fn select_path<'a, T, U: AsRef<str>>(&mut self, path: &'a T) -> Res
    where
        &'a T: IntoIterator<Item = U>,
    {
        self.try_select_path(path)?.ok_or_else(|| {
            let owned = path.into_iter().map(|s| s.as_ref().to_owned()).collect();
            StreamError::PathNotFound(owned)
        })
    }

    pub fn close_dict(&mut self) -> Res {
        self.skip_to_depth(self.depth - 1)
    }

    fn skip_to_depth(&mut self, target_depth: i64) -> Res {
        while self.depth > target_depth { 
            self.expect_next()?;
        }
        Ok(())
    }
}
