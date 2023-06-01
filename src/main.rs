use std::cmp::Ordering;
use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use indexmap::IndexMap;
use std::io::Read;
use std::io::Write;

#[derive(Copy, Clone, Eq, PartialEq, Ord)]
struct FactorioVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}

impl Ord for FactorioVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major == other.major {
            if self.minor == other.minor {
                if self.patch == other.patch {
                    if self.build == other.build { Some(Ordering::Equal)                    }
                    else if self.build > other.build { Some(Ordering::Greater) }
                    else if self.build < other.build { Some(Ordering::Less) }
                }
                else if self.patch > other.patch { Some(Ordering::Greater) }
                else if self.patch < other.patch { Some(Ordering::Less) }
            }
            else if self.minor > other.minor { Some(Ordering::Greater) }
            else if self.minor < other.minor { Some(Ordering::Less) }
        }
        else if self.major > other.major { Some(Ordering::Greater) }
        else if self.major < other.major { Some(Ordering::Less) }
    }
}

impl Codec for FactorioVersion {
    fn decode(input: &mut impl Read) -> anyhow::Result<FactorioVersion> {
        let [major, minor, patch, build] = {
            let mut vers = [0; 4];
            input.read_u16_into::<LE>(&mut vers)?;
            vers
        };
        Ok(FactorioVersion {
            major,
            minor,
            patch,
            build,
        })
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_u16::<LE>(self.major)?;
        writer.write_u16::<LE>(self.minor)?;
        writer.write_u16::<LE>(self.patch)?;
        writer.write_u16::<LE>(self.build)?;
        Ok(())
    }
}

struct Property {
    pub any_flag: bool,
    pub value: PropertyValue,
}

enum PropertyValue {
    None,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Property>),
    Dictionary(IndexMap<String, Property>),
}

impl Codec for Property {
    fn decode(input: &mut impl Read) -> anyhow::Result<Property> {
        let [vtype, any_flag] = {
            let mut tree_header = [0; 2];
            input.read_exact(&mut tree_header)?;
            tree_header
        };
        let value = match vtype {
            0 => PropertyValue::None,
            1 => PropertyValue::Bool(Codec::decode(input)?),
            2 => PropertyValue::Number(Codec::decode(input)?),
            3 => PropertyValue::String(Codec::decode(input)?),
            4 => PropertyValue::List(Codec::decode(input)?),
            5 => PropertyValue::Dictionary(Codec::decode(input)?),
            _ => return Err(anyhow!("Unknown type")),
        };
        Ok(Property {
            any_flag: loose_bool(any_flag),
            value,
        })
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

struct Settings {
    pub version: FactorioVersion,
    pub settings: Property,
}

impl Codec for Settings {
    fn decode(_input: &mut impl Read) -> anyhow::Result<Settings> {
        todo!()
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}

trait Codec: Sized {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self>;
    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()>;
}

impl Codec for bool {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        reader
            .read_u8()
            .map(loose_bool)
            .map_err(anyhow::Error::from)
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

impl Codec for f64 {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        Ok(reader.read_f64::<LE>()?)
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

impl Codec for String {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        let empty_byte = reader.read_u8()?;
        if !loose_bool(empty_byte) {
            // if not empty
            let length = read_optimized_u32(reader)?;
            let mut vec = vec![0; length as usize];
            reader.read_exact(&mut vec[..])?;
            Ok(String::from_utf8(vec)?)
        } else {
            Ok(String::new())
        }
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

impl Codec for Vec<Property> {
    fn decode(_reader: &mut impl Read) -> anyhow::Result<Self> {
        todo!()
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

impl Codec for IndexMap<String, Property> {
    fn decode(_reader: &mut impl Read) -> anyhow::Result<Self> {
        todo!()
    }

    fn encode(&self, _writer: &mut impl Write) -> anyhow::Result<()> {
        todo!()
    }
}

#[inline]
const fn loose_bool(input: u8) -> bool {
    matches!(input, 1)
}

#[inline]
fn read_optimized_u32(reader: &mut impl Read) -> anyhow::Result<u32> {
    Ok(match reader.read_u8()? {
        0xff => reader.read_u32::<LE>()?,
        byte => byte as u32,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use hex_literal::hex;
    use crate::{Codec, FactorioVersion};

    #[test]
    fn debug() {
        let data = hex!("01 00 01 00 52 00 04 00 00 05 00 03 00 00 00 00 07 73 74 61 72 74 75 70");
        let mut cursor = Cursor::new(data);
        let a = FactorioVersion::decode(&mut cursor).expect("factorio version");
    }
}