mod codec;
mod types;

use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::fs::read;
use std::io::Read;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::codec::Settings;


fn main() {
    println!("Hello, world!");
}

fn flatten(settings: &Settings) -> Settings {
    // let version = settings.version;
    // let new_properties
    // Settings {
    //     version,
    //     properties: settings
    // }
    todo!()
}
