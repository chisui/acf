use std::io::{Write, Result};
use std::str;


#[derive(Clone, Copy)]
pub enum JsonWriterCfg {
    Compact,
    Pretty {
        indent: u8,
    },
}

pub struct JsonWriter<W> {
    pub out: W,
    pub cfg: JsonWriterCfg,
    pub depth: u8,
}
impl<W: Write> JsonWriter<W> {
    pub fn new(cfg: JsonWriterCfg, out: W) -> Self {
        Self { cfg, out, depth: 0 }
    }
    fn newline(&mut self) -> Result<()> {
        if let JsonWriterCfg::Pretty { indent } = self.cfg {
            write!(self.out, "\n")?;
            for _ in 0..(self.depth as u32 * indent as u32) {
                write!(self.out, " ")?;
            }
        }
        Ok(())
    }
    fn write_string<S: AsRef<str>>(&mut self, s: S) -> Result<()> {
        write!(self.out, "\"{}\"", s.as_ref())
    }
    pub fn string_value<S: AsRef<str>>(&mut self, s: S) ->  Result<()> {
        self.write_string(s)
    }
    pub fn begin_obj(&mut self) -> Result<()> {
        self.depth += 1;
        write!(self.out, "{}", '{')
    }
    pub fn end_obj(&mut self) -> Result<()> {
        self.depth -= 1;
        self.newline()?;
        write!(self.out, "{}", '}')
    }
    pub fn begin_field<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.newline()?;
        self.write_string(name)?;
        write!(self.out, ":")?;
        match self.cfg {
            JsonWriterCfg::Pretty {..} => write!(self.out, " "),
            _ => Ok(()),
        }
    }
    pub fn end_field(&mut self) -> Result<()> {
        write!(self.out, ",")
    }
}
