use std::{
    io::{self, Write, Error, ErrorKind},
    fs::File,
    path::PathBuf,
};
use clap::Clap;
use steamacf::{AcfTokenStream};
mod json;
use crate::json::JsonWriterCfg;


#[derive(Debug, Clap, Clone, Copy)]
struct FormatArgs {
    /// compact instead of pretty-printed output
    #[clap(short, long)]
    compact: bool,

    /// how many spaces should be used per indentation step
    #[clap(short, long, default_value="2")]
    indent: u8,

}
impl From<FormatArgs> for JsonWriterCfg {
    fn from(args: FormatArgs) -> Self {
        if args.compact {
            JsonWriterCfg::Compact
        } else {
            JsonWriterCfg::Pretty {
                indent: args.indent,
            }
        }
    }
}

#[derive(Debug, Clap, Clone)]
#[clap(
    name = "acf",
    about = "converts steam .acf files to JSON"
)]
struct AcfArgs {
    #[clap(flatten)]
    format: FormatArgs,
    
    #[clap(parse(from_os_str))]
    file: PathBuf,
}


fn main() -> Result<(), Error> {
    let args = AcfArgs::parse();
    let f = File::open(args.file)?;
    let tokens = AcfTokenStream::new(f);
    let cfg = JsonWriterCfg::from(args.format);
    let mut out = io::stdout();
    json::pipe_to_json(cfg, tokens, &mut out)
        .map_err(|err| io::Error::new(ErrorKind::Other, err))?;
    out.write_all(b"\n")
}
