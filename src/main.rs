use crate::args::{Args, Format, Mode};
use crate::simple::ModSettings;
use anyhow::Context;
use either::Either;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

mod args;
mod codec;
mod simple;
mod types;

fn extension_is(path: &Path, s: &str) -> bool {
    path.extension()
        .map(|a| a.eq_ignore_ascii_case(s))
        .unwrap_or(false)
}

fn infer_args_mode(arg: &Args) -> Option<Mode> {
    if let Some(path) = arg.output.as_ref() {
        let json = extension_is(path, "json");
        let toml = extension_is(path, "toml");
        let dat = extension_is(path, "dat");
        if json || toml {
            Some(Mode::Decode)
        } else if dat {
            Some(Mode::Encode)
        } else {
            None
        }
    } else {
        let path = arg.input.as_path();
        let json = extension_is(path, "json");
        let toml = extension_is(path, "toml");
        let dat = extension_is(path, "dat");
        if json || toml {
            Some(Mode::Encode)
        } else if dat {
            Some(Mode::Decode)
        } else {
            None
        }
    }
}

fn infer_args_format(arg: &Args, mode: &Mode) -> Option<Format> {
    match mode {
        Mode::Encode => {
            let path = arg.input.as_path();
            let json = extension_is(path, "json");
            let toml = extension_is(path, "toml");
            if json {
                Some(Format::Json)
            } else if toml {
                Some(Format::Toml)
            } else {
                None
            }
        }
        Mode::Decode => arg.output.as_deref().and_then(|path| {
            let json = extension_is(path, "json");
            let toml = extension_is(path, "toml");
            if json {
                Some(Format::Json)
            } else if toml {
                Some(Format::Toml)
            } else {
                None
            }
        }),
    }
}

fn main() -> anyhow::Result<()> {
    let arg = args::parse_args();
    let mode = match arg.mode {
        Some(mode) => mode,
        None => {
            infer_args_mode(&arg).ok_or(anyhow::anyhow!("Unable to infer mode from arguments"))?
        }
    };
    let format = match arg.format {
        Some(format) => format,
        None => infer_args_format(&arg, &mode)
            .ok_or(anyhow::anyhow!("Unable to infer format from arguments"))?,
    };
    let mut input_reader = if matches!(arg.input.to_str(), Some("-")) {
        BufReader::new(Either::Left(std::io::stdin().lock()))
    } else {
        BufReader::new(Either::Right(
            File::open(arg.input).context("Opening input file")?,
        ))
    };
    let mut output_writer = if let Some(output) = arg.output {
        BufWriter::new(Either::Left(File::create(output)?))
    } else {
        BufWriter::new(Either::Right(std::io::stdout().lock()))
    };

    match mode {
        Mode::Encode => encode(format, &mut input_reader, &mut output_writer)?,
        Mode::Decode => decode(format, &mut input_reader, &mut output_writer)?,
    }

    Ok(())
}

fn decode(format: Format, reader: &mut impl Read, writer: &mut impl Write) -> anyhow::Result<()> {
    let decoded = codec::Settings::from_reader(reader).context("Decoding settings")?;
    let settings = simple::ModSettings::try_from(&decoded).context("Converting format")?;
    let serialized = match format {
        Format::Toml => toml::to_string_pretty(&settings).context("Serializing to TOML")?,
        Format::Json => serde_json::to_string_pretty(&settings).context("Serializing to JSON")?,
    };

    writer
        .write_all(serialized.as_bytes())
        .context("Writing output")
}

fn encode(format: Format, reader: &mut impl Read, writer: &mut impl Write) -> anyhow::Result<()> {
    let mut data = String::new();
    reader.read_to_string(&mut data).context("Reading stream")?;
    let deserialized: ModSettings = match format {
        Format::Toml => toml::from_str(&data).context("Deserializing TOML")?,
        Format::Json => serde_json::from_str(&data).context("Deserializing JSON")?,
    };

    codec::Settings::from_simple(&deserialized)
        .encode_to_writer(writer)
        .context("Encoding settings")
}
