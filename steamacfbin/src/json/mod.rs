use std::io::{Write, Error, ErrorKind};
use steamacf::{AcfToken, StreamError, Res};
mod writer;
use crate::json::writer::JsonWriter;

pub use crate::json::writer::JsonWriterCfg;


pub fn pipe_to_json<I: Iterator<Item = Res<AcfToken>>, W: Write>(cfg: JsonWriterCfg, iter: I, out: W) -> Res<()> {
    Pipe {
        iter,
        out: JsonWriter::new(cfg, out),
    }.write_object()
}
struct Pipe<I, W> {
    iter: I,
    out: JsonWriter<W>,
}
impl<I: Iterator<Item = Res<AcfToken>>, W: Write> Pipe<I, W> {
    fn write_object(&mut self) -> Res<()> {
        self.out.begin_obj()?;
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
                self.out.end_field()?;
            } else {
                is_not_first = true;
            }
            self.out.begin_field(n)?;
            let v = self.iter.next()
                .ok_or(Error::new(ErrorKind::UnexpectedEof, "expected value"))?;
            match v? {
                AcfToken::String(s) => self.out.string_value(s)
                    .map_err(StreamError::from),
                AcfToken::DictStart => self.write_object()
                    .map_err(StreamError::from),
                t => Err(StreamError::UnexpectedToken(t)),
            }?;
        }
        self.out.end_obj()
            .map_err(StreamError::from)
    }
}