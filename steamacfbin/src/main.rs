use std::io::{self, Result};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use steamacf::AcfTokenStream;
mod json;
use crate::json::JsonWriter;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "acf",
    about = "converts steam .acf files to JSON"
)]
struct AcfArgs {
    /// compact instead of pretty-printed output
    #[structopt(short, long)]
    compact: bool,

    /// how many spaces should be used per indentation step
    #[structopt(short, long, default_value="2")]
    indent: u32,

    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = AcfArgs::from_args();
    let f = File::open(args.file)?;
    let tokens = AcfTokenStream::new(f);
    let w = JsonWriter {
        compact: args.compact,
        indent: args.indent,
    };
    w.write(tokens, io::stdout())
}
