# factorio-settings

A command line utility to decode and encode mod settings (`mod-settings.dat`) for Factorio

---

## Features
- Easy-to-use command line interface
- Byte-parity with Factorio (so far so good)
- Available formats for decoded representation: JSON, TOML

## Quick Examples
```sh
factorio-settings mod-settings.dat -f json | jq '.startup["my-color-setting"].value.g=1' | factorio-settings -f json - mod-settings.dat
factorio-settings mod-settings.dat json_settings.json
factorio-settings mod-settings.dat toml_settings.toml
```
## Usage
```
Usage: factorio-settings [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   The input path to read binary settings from. Use "-" for stdin
  [OUTPUT]  The output file. Overwrites if present. Stdout if omitted

Options:
  -m, --mode <MODE>      Whether to encode or decode the input. If not provided, will attempt to infer based on output type, or input type, in that order [possible values: decode, encode]
  -f, --format <FORMAT>  The format for the serialized input/output. If omitted, will attempt to infer based on mode and input or output [possible values: toml, json]
  -h, --help             Print help
  -V, --version          Print version
```