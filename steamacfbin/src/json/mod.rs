use std::io::Write;
use steamacf::{AcfToken, StreamError, ParseError};
mod writer;
use crate::json::writer::JsonWriter;

pub use crate::json::writer::JsonWriterCfg;


pub fn pipe_to_json<I, W>(cfg: JsonWriterCfg, iter: I, out: W) -> Result<(), StreamError> 
where
    I: Iterator<Item = Result<AcfToken, ParseError>>,
    W: Write,
{
    Pipe {
        iter,
        out: JsonWriter::new(cfg, out),
    }.write_object()
}
struct Pipe<I, W> {
    iter: I,
    out: JsonWriter<W>,
}
impl<I: Iterator<Item = Result<AcfToken, ParseError>>, W: Write> Pipe<I, W> {
    fn write_object(&mut self) -> Result<(), StreamError> {
        self.out.begin_obj()
            .map_err(ParseError::from)
            .map_err(StreamError::from)?;
        let mut is_not_first = false;
        loop {
            let t = match self.iter.next() {
                None => { break; },
                Some(t) => t,
            }?;
            let n = match t {
                AcfToken::DictEnd => { break; }
                AcfToken::String(n) => Ok(n),
                t => Err(StreamError::UnexpectedToken(t)),
            }?;
            if is_not_first {
                self.out.end_field()
                    .map_err(ParseError::from)
                    .map_err(StreamError::from)?;
            } else {
                is_not_first = true;
            }
            self.out.begin_field(n)
                .map_err(ParseError::from)
                .map_err(StreamError::from)?;
            let v = self.iter.next()
                .ok_or(StreamError::from(ParseError::UnexpectedEof))?;
            match v? {
                AcfToken::String(s) => self.out.string_value(s)
                    .map_err(ParseError::from)
                    .map_err(StreamError::from),
                AcfToken::DictStart => self.write_object(),
                t => Err(StreamError::UnexpectedToken(t)),
            }?;
        }
        self.out.end_obj()
            .map_err(ParseError::from)
            .map_err(StreamError::from)
    }
}