use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::codec::{Property, PropertyValue, Settings};
use crate::types::FactorioVersion;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModSettings {
    pub factorio_version: FactorioVersion,
    pub startup: IndexMap<String, ModSettingsValue>,
    #[serde(rename = "runtime-global")]
    pub runtime_global: IndexMap<String, ModSettingsValue>,
    #[serde(rename = "runtime-per-user")]
    pub runtime_per_user: IndexMap<String, ModSettingsValue>,
}

fn property_map_parse(root: &IndexMap<String, Property>, key: &str) -> Result<IndexMap<String, ModSettingsValue>, anyhow::Error> {
    let map = root.get(key)
        .ok_or(anyhow::anyhow!("Missing {} settings", key))?
        .value.as_dictionary()
        .ok_or(anyhow::anyhow!("{} settings is not a dictionary", key))?;
    map.iter().map(|(key, value)| {
        ModSettingsValue::try_from(value)
            .map(|a| (key.clone(), a))
    }).collect::<Result<IndexMap<_, _>, _>>()
}

impl TryFrom<&Settings> for ModSettings {
    type Error = anyhow::Error;

    fn try_from(value: &Settings) -> Result<Self, Self::Error> {

        let root = value.properties.value
            .as_dictionary()
            .ok_or(anyhow::anyhow!("Main properties is not a dictionary"))?;
        let startup = property_map_parse(root, "startup")?;
        let runtime_global = property_map_parse(root, "runtime-global")?;
        let runtime_per_user = property_map_parse(root, "runtime-per-user")?;
        Ok(Self { factorio_version: value.version.clone(), startup, runtime_global, runtime_per_user })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type", content = "value")]
pub enum ModSettingsValue {
    None,
    Bool(bool),
    Number(f64),
    String(String),
    Color {
        r: f64,
        g: f64,
        b: f64,
        a: f64,
    },
}

impl TryFrom<&Property> for ModSettingsValue {
    type Error = anyhow::Error;

    fn try_from(value: &Property) -> Result<Self, Self::Error> {
        match &value.value {
            PropertyValue::Dictionary(dict) => {
                let value = dict.get("value").ok_or(anyhow::anyhow!("Mod setting dictionary missing value property"))?;
                match &value.value {
                    PropertyValue::Bool(b) => Ok(ModSettingsValue::Bool(*b)),
                    PropertyValue::Number(n) => Ok(ModSettingsValue::Number(*n)),
                    PropertyValue::String(s) => Ok(ModSettingsValue::String(s.clone())),
                    PropertyValue::Dictionary(dict) => {
                        let r = *dict.get("r")
                            .ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - missing r (red) value: {:?}", dict))?
                            .value.as_number().ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - r (red) value is not number"))?;
                        let g = *dict.get("r")
                            .ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - missing g (green) value: {:?}", dict))?
                            .value.as_number().ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - g (green) value is not number"))?;
                        let b = *dict.get("r")
                            .ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - missing b (blue) value: {:?}", dict))?
                            .value.as_number().ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - b (blue) value is not number"))?;
                        let a = *dict.get("r")
                            .ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - missing a (alpha) value: {:?}", dict))?
                            .value.as_number().ok_or(anyhow::anyhow!("Mod setting value is dictionary - assuming color - a (alpha) value is not number"))?;
                        Ok(ModSettingsValue::Color { r, g, b, a })
                    },
                    b => Err(anyhow::anyhow!("Mod setting value: Invalid type for value parameter: {:?}", b))
                }
            },
            _ => Err(anyhow::anyhow!("Mod setting should be a dictionary"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Write};
    use indexmap::IndexMap;
    use crate::codec;
    use crate::types::FactorioVersion;
    use super::ModSettings;

    #[test]
    fn serialize_empty() {
        let settings = ModSettings {
            factorio_version: FactorioVersion { major: 1, minor: 1, build: 4, patch: 82 },
            startup: IndexMap::new(),
            runtime_global: IndexMap::new(),
            runtime_per_user: IndexMap::new(),
        };
        let pretty = serde_json::to_string_pretty(&settings).expect("writing");
        println!("{}", &pretty);
    }

    fn load_complex_settings() -> ModSettings {
        let mut file = BufReader::new(File::open("test_data/complex-settings.dat").expect("loading complex settings"));
        let settings = codec::Settings::from_reader(&mut file).expect("parsing complex settings");
        ModSettings::try_from(&settings).expect("Simplifying complex settings")
    }

    #[test]
    fn simplify_complex_settings() {
        load_complex_settings();
    }

    #[test]
    fn serialize_complex_json() {
        let settings = load_complex_settings();
        serde_json::to_writer_pretty(BufWriter::new(File::create("test_output/simplified-complex.json").expect("creating output file")), &settings).expect("serializing");
    }

    #[test]
    fn serialize_complex_toml() {
        let settings = load_complex_settings();
        let s_toml = toml::to_string_pretty(&settings).expect("serializing");
        let mut file = BufWriter::new(File::create("test_output/simplified-complex.toml").expect("creating output file"));
        file.write_all(s_toml.as_bytes()).expect("Writing output file");
    }

    #[test]
    fn serialize_deserialize_parity() {
        let settings = load_complex_settings();
        let s_json = serde_json::to_string_pretty(&settings).expect("serializing json");
        let s_toml = toml::to_string_pretty(&settings).expect("serializing toml");

        let json_settings: ModSettings = serde_json::from_str(&s_json).expect("Deserializing json");
        let toml_settings: ModSettings = toml::from_str(&s_toml).expect("Deserializing toml");

        assert_eq!(&settings, &json_settings, "Json settings equal");
        assert_eq!(&settings, &toml_settings, "Toml settings equal");
        assert_eq!(&json_settings, &toml_settings, "Json Toml settings equal each other");
    }
}
