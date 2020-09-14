use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::str::FromStr;

use dialoguer::{Input, MultiSelect, Select};

//use textwrap::{Wrapper, hyphenation::{Language, Load, Standard}};
use textwrap::{termwidth, Wrapper};

use serde::{de, Deserialize, Deserializer};
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

    #[serde(deserialize_with = "de_int_key")]
    Bitmask(BTreeMap<i64, String>),
    #[serde(deserialize_with = "de_int_key")]
    Values(BTreeMap<i64, String>),
}

#[derive(Debug, Clone, Deserialize)]
pub enum User {
    Standard,
    Advanced,
    User,
}

struct Selection(i64, String);

impl Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(f)
    }
}

impl Parameter {
    pub fn edit(&self, current_value: f32) -> f32 {
        match &self.data {
            None => {
                let mut input = Input::new();
                input
                    .with_initial_text(current_value.to_string())
                    .with_prompt(&self.name);
                input.interact().unwrap_or(current_value)
            }
            Some(DataType::Range { high, low }) => {
                let mut input = Input::new();
                input
                    .with_initial_text(current_value.to_string())
                    .with_prompt(format!("{} [{} {}]", &self.name, low, high));
                input.interact().unwrap_or(current_value)
            }
            Some(DataType::Values(values)) => {
                let mut items: Vec<_> = values
                    .clone()
                    .into_iter()
                    .map(|(k, v)| Selection(k, v))
                    .collect();
                items.push(Selection(0, String::from("Enter a Custom value")));
                let mut select = Select::new();
                select.items(&items).with_prompt(&self.name);
                if let Some(index) = items.iter().position(|x| x.0 as f32 == current_value) {
                    select.default(index);
                }

                match select.interact_opt() {
                    // user wants to enter a custom value
                    Ok(Some(selection)) if selection == items.len() - 1 => {
                        let mut input = Input::new();
                        input
                            .with_initial_text(current_value.to_string())
                            .with_prompt(format!("{}", &self.name));
                        input.interact().unwrap_or(current_value)
                    }
                    // user chose one of the provided values
                    Ok(Some(selection)) => items[selection].0 as f32,
                    // something went wrong, don't change anything
                    _ => current_value,
                }
            }
            Some(DataType::Bitmask(values)) => {
                let mut items: Vec<_> = values
                    .clone()
                    .into_iter()
                    .map(|(k, v)| Selection(k, v))
                    .collect();
                items.push(Selection(0, String::from("Enter a Custom value")));
                let mut select = MultiSelect::new();
                select.items(&items).with_prompt(&self.name);

                match select.interact() {
                    // user wants to enter a custom value
                    Ok(selection) if selection.contains(&(items.len() - 1)) => {
                        let mut input = Input::new();
                        input
                            .with_initial_text(current_value.to_string())
                            .with_prompt(format!("{}", &self.name));
                        input.interact().unwrap_or(current_value)
                    }
                    // user chose some of the provided values
                    Ok(selection) => {
                        let mut bytes: i64 = 0;
                        for s in selection {
                            bytes |= 1 << items[s].0;
                        }
                        bytes as f32
                    }
                    _ => current_value,
                }
            }

            _ => current_value,
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn de_int_key<'de, D, K, V>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Eq + FromStr + std::hash::Hash + std::cmp::Ord,
    K::Err: Display,
    V: Deserialize<'de>,
{
    let string_map = <BTreeMap<String, V>>::deserialize(deserializer)?;
    let mut map = BTreeMap::new();
    for (s, v) in string_map {
        let k = K::from_str(&s).map_err(de::Error::custom)?;
        map.insert(k, v);
    }
    Ok(map)
}
