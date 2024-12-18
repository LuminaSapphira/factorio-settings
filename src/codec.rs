use crate::simple::{ModSettings, ModSettingsValue};
use crate::types::FactorioVersion;
use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use indexmap::IndexMap;
use std::io::{Read, Write};

const TYPE_NONE: u8 = 0;
const TYPE_BOOL: u8 = 1;
const TYPE_DOUBLE: u8 = 2;
const TYPE_STRING: u8 = 3;
const TYPE_LIST: u8 = 4;
const TYPE_DICTIONARY: u8 = 5;
const TYPE_INTEGER: u8 = 6;

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

#[derive(Clone, Debug)]
pub struct Property {
    pub any_flag: bool,
    pub value: PropertyValue,
}

#[derive(Clone, Debug)]
pub enum PropertyValue {
    None,
    Bool(bool),
    Double(f64),
    String(String),
    List(Vec<Property>),
    Dictionary(IndexMap<String, Property>),
    Integer(i64),
}

impl PropertyValue {
    #[allow(unused)]
    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_double(&self) -> Option<&f64> {
        match self {
            Self::Double(f) => Some(f),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_list(&self) -> Option<&Vec<Property>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_dictionary(&self) -> Option<&IndexMap<String, Property>> {
        match self {
            Self::Dictionary(map) => Some(map),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_integer(&self) -> Option<&i64> {
        match self {
            Self::Integer(i) => Some(i),
            _ => None,
        }
    }
}

impl Codec for Property {
    fn decode(input: &mut impl Read) -> anyhow::Result<Property> {
        let [vtype, any_flag] = {
            let mut tree_header = [0; 2];
            input.read_exact(&mut tree_header)?;
            tree_header
        };
        let value = match vtype {
            TYPE_NONE => PropertyValue::None,
            TYPE_BOOL => PropertyValue::Bool(Codec::decode(input)?),
            TYPE_DOUBLE => PropertyValue::Double(Codec::decode(input)?),
            TYPE_STRING => PropertyValue::String(Codec::decode(input)?),
            TYPE_LIST => PropertyValue::List(Codec::decode(input)?),
            TYPE_DICTIONARY => PropertyValue::Dictionary(Codec::decode(input)?),
            TYPE_INTEGER => PropertyValue::Integer(Codec::decode(input)?),
            other => return Err(anyhow!("Unknown type: {:#x}", other)),
        };
        Ok(Property {
            any_flag: loose_bool(any_flag),
            value,
        })
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        match &self.value {
            PropertyValue::None => {
                writer.write_u8(TYPE_NONE)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
            }
            PropertyValue::Bool(b) => {
                writer.write_u8(TYPE_BOOL)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                b.encode(writer)?;
            }
            PropertyValue::Double(num) => {
                writer.write_u8(TYPE_DOUBLE)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                num.encode(writer)?;
            }
            PropertyValue::String(string) => {
                writer.write_u8(TYPE_STRING)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                string.encode(writer)?;
            }
            PropertyValue::List(list) => {
                writer.write_u8(TYPE_LIST)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                list.encode(writer)?;
            }
            PropertyValue::Dictionary(dict) => {
                writer.write_u8(TYPE_DICTIONARY)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                dict.encode(writer)?;
            }
            PropertyValue::Integer(int) => {
                writer.write_u8(TYPE_INTEGER)?;
                writer.write_u8(loose_bool_byte(self.any_flag))?;
                int.encode(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub version: FactorioVersion,
    pub properties: Property,
}

impl Settings {
    pub fn from_reader(reader: &mut impl Read) -> anyhow::Result<Settings> {
        Self::decode(reader)
    }

    pub fn encode_to_writer(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        self.encode(writer)
    }

    fn convert_simple_index_map(map: &IndexMap<String, ModSettingsValue>) -> Property {
        let mut properties = IndexMap::with_capacity(map.len());
        for (key, value) in map {
            let prop_value = match value {
                ModSettingsValue::None => PropertyValue::None,
                ModSettingsValue::Bool(b) => PropertyValue::Bool(*b),
                ModSettingsValue::Double(f) => PropertyValue::Double(*f),
                ModSettingsValue::String(s) => PropertyValue::String(s.clone()),
                ModSettingsValue::Color { r, g, b, a } => {
                    let mut color_map = IndexMap::with_capacity(4);
                    color_map.insert(
                        "r".to_owned(),
                        Property {
                            any_flag: false,
                            value: PropertyValue::Double(*r),
                        },
                    );
                    color_map.insert(
                        "g".to_owned(),
                        Property {
                            any_flag: false,
                            value: PropertyValue::Double(*g),
                        },
                    );
                    color_map.insert(
                        "b".to_owned(),
                        Property {
                            any_flag: false,
                            value: PropertyValue::Double(*b),
                        },
                    );
                    color_map.insert(
                        "a".to_owned(),
                        Property {
                            any_flag: false,
                            value: PropertyValue::Double(*a),
                        },
                    );
                    PropertyValue::Dictionary(color_map)
                }
                ModSettingsValue::Integer(i) => PropertyValue::Integer(*i),
            };
            let mut inner_props_map = IndexMap::with_capacity(1);
            inner_props_map.insert(
                "value".to_owned(),
                Property {
                    any_flag: false,
                    value: prop_value,
                },
            );
            properties.insert(
                key.clone(),
                Property {
                    any_flag: false,
                    value: PropertyValue::Dictionary(inner_props_map),
                },
            );
        }
        Property {
            any_flag: false,
            value: PropertyValue::Dictionary(properties),
        }
    }

    pub fn from_simple(simple: &ModSettings) -> Settings {
        let startup_properties = Self::convert_simple_index_map(&simple.startup);
        let runtime_properties = Self::convert_simple_index_map(&simple.runtime_global);
        let runtime_per_user_properties = Self::convert_simple_index_map(&simple.runtime_per_user);

        let mut root_map = IndexMap::new();
        root_map.insert("startup".to_owned(), startup_properties);
        root_map.insert("runtime-global".to_owned(), runtime_properties);
        root_map.insert("runtime-per-user".to_owned(), runtime_per_user_properties);

        let root = Property {
            any_flag: false,
            value: PropertyValue::Dictionary(root_map),
        };
        Settings {
            properties: root,
            version: simple.factorio_version,
        }
    }
}

impl Codec for Settings {
    fn decode(input: &mut impl Read) -> anyhow::Result<Settings> {
        let version = FactorioVersion::decode(input)?;
        if input.read_u8()? != 0 {
            return Err(anyhow!("Byte at 0x8 should be false"));
        }
        let settings = Property::decode(input)?;
        Ok(Self {
            version,
            properties: settings,
        })
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        self.version.encode(writer)?;
        writer.write_u8(0)?;
        self.properties.encode(writer)?;
        Ok(())
    }
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

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl Codec for f64 {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        Ok(reader.read_f64::<LE>()?)
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_f64::<LE>(*self)?;
        Ok(())
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

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        // if self.is_empty() { writer.write_u8(1)?; }
        // else { writer.write_u8(0)?; }
        writer.write_u8(0)?;
        write_optimized_u32(writer, self.len() as u32)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
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
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        let count = reader.read_u32::<LE>()?;
        let mut map = IndexMap::with_capacity(count as usize);
        for _ in 0..count {
            let name = String::decode(reader)?;
            let value = Property::decode(reader)?;
            map.insert(name, value);
        }
        Ok(map)
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_u32::<LE>(self.len() as u32)?;
        for (key, value) in self {
            key.encode(writer)?;
            value.encode(writer)?;
        }
        Ok(())
    }
}

impl Codec for i64 {
    fn decode(reader: &mut impl Read) -> anyhow::Result<Self> {
        Ok(reader.read_i64::<LE>()?)
    }

    fn encode(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writer.write_i64::<LE>(*self)?;
        Ok(())
    }
}

#[inline]
const fn loose_bool(input: u8) -> bool {
    matches!(input, 1)
}

#[inline]
const fn loose_bool_byte(input: bool) -> u8 {
    input as u8
}

#[inline]
fn read_optimized_u32(reader: &mut impl Read) -> anyhow::Result<u32> {
    Ok(match reader.read_u8()? {
        0xff => reader.read_u32::<LE>()?,
        byte => byte as u32,
    })
}

#[inline]
fn write_optimized_u32(writer: &mut impl Write, value: u32) -> anyhow::Result<()> {
    if value < 0xff {
        writer.write_u8(value as u8)?;
    } else {
        writer.write_u8(0xff)?;
        writer.write_u32::<LE>(value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Codec, Property, PropertyValue, Settings};
    use crate::simple::ModSettings;
    use crate::types::FactorioVersion;
    use hex_literal::hex;
    use indexmap::IndexMap;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};
    use std::path::Path;

    #[test]
    fn simple_encoded() {
        let data = hex!("01 00 01 00 52 00 04 00 00 05 00 03 00 00 00 00 07 73 74 61 72 74 75 70 05 00 01 00 00 00 00 11 6D 79 2D 73 74 72 69 6E 67 2D 73 65 74 74 69 6E 67 05 00 01 00 00 00 00 05 76 61 6C 75 65 03 00 00 08 64 65 61 64 62 65 65 66 00 0E 72 75 6E 74 69 6D 65 2D 67 6C 6F 62 61 6C 05 00 00 00 00 00 00 10 72 75 6E 74 69 6D 65 2D 70 65 72 2D 75 73 65 72 05 00 00 00 00 00");
        let mut cursor = Cursor::new(data);
        let settings = Settings::decode(&mut cursor).expect("decoding settings");
        assert_eq!(
            settings.version,
            FactorioVersion {
                major: 1,
                minor: 1,
                patch: 82,
                build: 4
            },
            "version"
        );
        assert!(!settings.properties.any_flag, "should be false");
        println!("{:?}", &settings.properties);
        let root = get_map(&settings.properties);
        let startup_dict = get_map(root.get("startup").expect("missing startup"));
        let my_setting = get_map(
            startup_dict
                .get("my-string-setting")
                .expect("missing my-string-setting"),
        );
        let value = my_setting.get("value").expect("missing value");
        match &value.value {
            PropertyValue::String(s) => assert_eq!(s, "deadbeef", "incorrect value"),
            _ => panic!("Incorrect type"),
        }
    }

    #[test]
    fn complex() {
        let mut reader =
            BufReader::new(File::open("test_data/complex-settings.dat").expect("opening file"));
        Settings::decode(&mut reader).expect("decoding settings");
    }

    #[test]
    fn decode_encode_parity_1_1() {
        decode_encode_parity("test_data/complex-settings.dat");
    }

    #[test]
    fn decode_encode_parity_2_0() {
        decode_encode_parity("test_data/settings-2.0.dat");
    }

    fn decode_encode_parity(file: impl AsRef<Path>) {
        let mut reader = BufReader::new(File::open(file).expect("opening file"));
        let data = {
            let mut vec = Vec::with_capacity(90000);
            reader.read_to_end(&mut vec).expect("Reading file");
            vec
        };
        let mut cursor = Cursor::new(&data);
        let settings = Settings::decode(&mut cursor).expect("Decoding settings");

        let encoded_data = {
            let vec = Vec::<u8>::with_capacity(data.capacity());
            let mut cursor = Cursor::new(vec);
            settings.encode(&mut cursor).expect("Encoding settings");
            cursor.into_inner()
        };
        assert_eq!(data, encoded_data);
    }

    #[test]
    fn complex_2_0() {
        let mut reader =
            BufReader::new(File::open("test_data/settings-2.0.dat").expect("opening file"));
        let set = Settings::decode(&mut reader).expect("decoding settings");
        ModSettings::try_from(&set).expect("to modsettings");
    }

    fn get_map(prop: &Property) -> &IndexMap<String, Property> {
        match &prop.value {
            PropertyValue::Dictionary(map) => map,
            _ => panic!("expected dictionary"),
        }
    }
}
