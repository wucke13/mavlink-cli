use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use serde::{Deserialize, Deserializer};
use serde_json::from_reader;
use skim::prelude::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Definitions {
    copter: BTreeMap<String, Parameter>,
    plane: BTreeMap<String, Parameter>,
    rover: BTreeMap<String, Parameter>,
    sub: BTreeMap<String, Parameter>,
    tracker: BTreeMap<String, Parameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    #[serde(default)]
    name: String,
    description: String,
    display_name: String,
    #[serde(default)]
    user: User,
    #[serde(flatten)]
    data: Option<DataType>,
    #[serde(default)]
    vehicle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DataType {
    Range {
        #[serde(deserialize_with = "de_from_str")]
        high: f32,
        #[serde(deserialize_with = "de_from_str")]
        low: f32,
        //increment: Option<f32>,
    },
    Bitmask(BTreeMap<String, String>),
    Values(BTreeMap<String, String>),
}

#[derive(Debug, Clone, Deserialize)]
pub enum User {
    Standard,
    Advanced,
    User,
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.display_name, self.description)
    }
}

impl SkimItem for Parameter {
    fn display(&self) -> Cow<AnsiString> {
        Cow::Owned(format!("{} [{}]", self.display_name, self.name).into())
    }

    fn text(&self) -> Cow<str> {
        let all_text = format!(
            "{}\n{}\n{}\n{}",
            self.name, self.display_name, self.description, self.vehicle
        );

        Cow::Owned(all_text)
    }

    fn preview(&self) -> ItemPreview {
        ItemPreview::AnsiText(format!("\x1b[31mhello:\x1b[m\n{}", self.description))
    }
}

impl Default for User {
    fn default() -> Self {
        User::Standard
    }
}

pub fn parse(path: &Path) -> io::Result<BTreeMap<String, Parameter>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let def: Definitions =
        from_reader(file).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let copter = def.copter.into_iter().map(|(k, mut v)| {
        v.vehicle = String::from("copter");
        v.name = k.clone();
        (k, v)
    });
    let plane = def.plane.into_iter().map(|(k, mut v)| {
        v.vehicle = String::from("plane");
        v.name = k.clone();
        (k, v)
    });
    let rover = def.rover.into_iter().map(|(k, mut v)| {
        v.vehicle = String::from("rover");
        v.name = k.clone();
        (k, v)
    });
    let sub = def.sub.into_iter().map(|(k, mut v)| {
        v.vehicle = String::from("sub");
        v.name = k.clone();
        (k, v)
    });
    let tracker = def.tracker.into_iter().map(|(k, mut v)| {
        v.vehicle = String::from("tracker");
        v.name = k.clone();
        (k, v)
    });

    Ok(copter
        .chain(plane)
        .chain(rover)
        .chain(sub)
        .chain(tracker)
        .collect())
}

fn de_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
