use std::path::PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Whether to encode or decode the input. If not provided, will attempt to infer based on output type, or input type, in that order.
    #[arg(short, long)]
    pub mode: Option<Mode>,
    /// The format for the serialized input/output. If omitted, will attempt to infer based on mode and input or output
    #[arg(short, long)]
    pub format: Option<Format>,
    /// The input path to read binary settings from. Use "-" for stdin
    pub input: PathBuf,
    /// The output file. Overwrites if present. Stdout if omitted.
    pub output: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Format {
    #[value(alias("t"))]
    Toml,
    #[value(alias("j"))]
    Json,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Mode {
    #[value(alias("d"))]
    Decode,
    #[value(alias("e"))]
    Encode,
}

pub fn parse_args() -> Args {
    Args::parse()
}
