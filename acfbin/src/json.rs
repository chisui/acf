use std::io::{Write, Error, ErrorKind, Result};
use steamacf::{AcfToken, UnexpectedToken};


#[derive(Clone, Copy)]
pub struct JsonWriter {
    pub compact: bool,
    pub indent: u32,
}
impl JsonWriter {
    pub fn write<I: Iterator<Item = Result<AcfToken>>, W: Write>(&self, iter: I, out: W) -> Result<()> {
        JsonWriterCtx {
            cfg: *self,
            depth: 0,
            iter,
            out,
        }.write_object()
    }
}

struct JsonWriterCtx<I: Iterator<Item = Result<AcfToken>>, W: Write> {
    cfg: JsonWriter,

    depth: u32,
    iter: I,
    out: W,
}
impl<I: Iterator<Item = Result<AcfToken>>, W: Write> JsonWriterCtx<I, W> {
    fn write_string(&self, s: String) -> Result<()> {
        print!("\"{}\"", s);
        Ok(())
    }
    fn newline(&mut self) -> Result<()> {
        if !self.cfg.compact {
            write!(self.out, "\n")?;
            for _ in 0..(self.depth * self.cfg.indent) {
                write!(self.out, " ")?;
            }
        }
        Ok(())
    }
    fn begin_obj(&mut self) -> Result<()> {
        self.depth += 1;
        write!(self.out, "{}", '{')
    }
    fn end_obj(&mut self) -> Result<()> {
        self.depth -= 1;
        self.newline()?;
        write!(self.out, "{}", '}')
    }
    fn begin_field(&mut self, name: String) -> Result<()> {
        self.newline()?;
        self.write_string(name)?;
        write!(self.out, ":")?;
        if !self.cfg.compact {
            write!(self.out, " ")?;
        }
        Ok(())
    }
    fn end_field(&mut self) -> Result<()> {
        write!(self.out, ",")
    }

    fn write_object(&mut self) -> Result<()> {
        self.begin_obj()?;
        let mut is_not_first = false;
        loop {
            let t = match self.iter.next() {
                None => { break; },
                Some(t) => t,
            }?;
            let n = match t {
                AcfToken::DictEnd => { break; }
                AcfToken::String(n) => Ok(n),
                t => Err(Error::new(ErrorKind::Other, UnexpectedToken(t))),
            }?;
            if is_not_first {
                self.end_field()?;
            } else {
                is_not_first = true;
            }
            self.begin_field(n)?;
            let v = self.iter.next()
                .ok_or(Error::new(ErrorKind::UnexpectedEof, "expected value"))?;
            match v? {
                AcfToken::String(s) => self.write_string(s),
                AcfToken::DictStart => self.write_object(),
                t => Err(Error::new(ErrorKind::Other, UnexpectedToken(t))),
            }?;
        }
        self.end_obj()
    }
}
