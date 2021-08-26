use std::io::{self, Read, Write, Error, ErrorKind, Result};
use std::str;
use std::fmt;
use std::error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;


#[derive(Clone, Debug)]
enum AcfToken {
    String(String),
    DictStart,
    DictEnd,
}
#[derive(Debug)]
struct UnexpectedCharacter(char);
impl error::Error for UnexpectedCharacter {}
impl fmt::Display for UnexpectedCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected Character '{:?}'", self.0)
    }
}
#[derive(Debug)]
struct UnexpectedToken(AcfToken);
impl error::Error for UnexpectedToken {}
impl fmt::Display for UnexpectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected Token {:?}", self.0)
    }
}

struct AcfTokenReader<R: Read>(R);
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
#[derive(Clone, Copy)]
struct JsonWriter {
    compact: bool,
    indent: u8,
}
impl JsonWriter {
    fn write<I: Iterator<Item = Result<AcfToken>>, W: Write>(&self, iter: I, out: W) -> Result<()> {
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

    depth: u8,
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


#[derive(Debug, StructOpt)]
#[structopt()]
struct AcfArgs {
    #[structopt(short, long)]
    compact: bool,

    #[structopt(short, long, default_value="2")]
    indent: u8,

    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = AcfArgs::from_args();
    let f = File::open(args.file)?;
    let tokens = AcfTokenReader(f);
    let w = JsonWriter {
        compact: args.compact,
        indent: args.indent,
    };
    w.write(tokens, io::stdout())
}
