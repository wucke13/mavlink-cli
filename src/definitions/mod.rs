use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use arc_swap::ArcSwap;
use once_cell::sync::Lazy;

use console::style;

use skim::{prelude::*, SkimItem};

use dialoguer::{Input, MultiSelect, Select};

use serde::{de, Deserialize, Deserializer};

mod ardupilot;

// Public API

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Definition {
    #[serde(default)]
    pub name: String,
    pub description: String,
    pub display_name: String,

    #[serde(default)]
    pub user: User,
    #[serde(flatten)]
    pub data: Option<DataType>,
    #[serde(default)]
    pub vehicle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DataType {
    Range {
        #[serde(deserialize_with = "de_from_str")]
        high: f32,
        #[serde(deserialize_with = "de_from_str")]
        low: f32,
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
    User, // TODO remove this, it is a bug
}

/// must be called once
pub fn init() {
    let ardupilot_included = include_str!("../../definitions/ArduPilot/result/apm.pdef.json");

    let ap = ardupilot::parse(ardupilot_included)
        .expect("parameters shipped inside binary do not parse. This is a bug. Please report it");

    DEFINITIONS.store(Arc::new(ap));

    // iterate over all (if any) provided search paths, try to parse parameter files
    for _path in std::env::var("MAVLINK_CLI_ARDUPILOT_PATH")
        .unwrap_or_default()
        .split(':')
        .map(|s| Path::new(s))
    {
        // TODO implement file level parser as well
    }

    // TODO implement the same for PX4
}

/// show information about a definiton
// TODO Return avoid cloning
pub fn lookup(param_name: &str) -> Option<Definition> {
    DEFINITIONS.load().get(param_name).cloned()
}

/// return all defintions
// TODO Return avoid cloning
pub fn all() -> Vec<Definition> {
    let mut result: Vec<_> = DEFINITIONS.load().values().cloned().collect();
    result.sort();
    result
}

/// Atomical Reference Counter which holding all currently known definitions.
/// This must not be mutated by concurrent code.
pub static DEFINITIONS: Lazy<ArcSwap<HashMap<String, Definition>>> =
    Lazy::new(|| ArcSwap::from_pointee(HashMap::new()));

// Implementation

impl Definition {
    /// interacts with the user, allowing a new value to be found
    pub fn interact(&self, current_value: f32) -> f32 {
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
                if let Some(index) = items
                    .iter()
                    .position(|x| (x.0 as f32 - current_value).abs() < 0.5)
                {
                    select.default(index);
                }

                match select.interact_opt() {
                    // user wants to enter a custom value
                    Ok(Some(selection)) if selection == items.len() - 1 => {
                        let mut input = Input::new();
                        input
                            .with_initial_text(current_value.to_string())
                            .with_prompt(self.name.to_string());
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
                            .with_prompt(self.name.to_string());
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
        }
    }
    pub fn name(&self) -> String {
        format!("{:-16}", style(&self.name).bold())
    }

    pub fn description(&self, width: usize) -> String {
        let title = style(&self.display_name).bold().underlined();
        let description = style(textwrap::fill(&self.description, width));

        let values = match &self.data {
            Some(DataType::Range { high, low }) => {
                format!("range: [{} - {}]", style(low).bold(), style(high).bold())
            }
            Some(DataType::Values(mapping)) | Some(DataType::Bitmask(mapping)) => {
                let max_key_length = mapping
                    .iter()
                    .map(|(k, _)| k.to_string().len())
                    .max()
                    .unwrap_or(0);
                let max_value_length = mapping.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
                let joint = " = ";
                let cols = width / (max_key_length + max_value_length + joint.len() + 1);
                let mut rows = mapping.len() / cols;
                if mapping.len() % cols != 0 {
                    rows += 1;
                }

                (0..rows)
                    .map(|initial_offset| {
                        mapping
                            .iter()
                            .skip(initial_offset)
                            .step_by(rows)
                            .enumerate()
                            .map(|(i, (k, v))| {
                                format!(
                                    "{}{:k_width$}{}{:^v_width$}",
                                    if i % cols == 0 { '\n' } else { ' ' },
                                    k,
                                    joint,
                                    v,
                                    k_width = max_key_length,
                                    v_width = max_value_length
                                )
                            })
                    })
                    .flatten()
                    .collect()
            }
            None => String::from(""),
        };

        format!("{}\n\n{}\n\n{}", title, description, values)
    }
}

impl SkimItem for Definition {
    fn display(&self) -> Cow<AnsiString> {
        Cow::Owned(AnsiString::parse(&self.name()))
    }

    fn text(&self) -> Cow<str> {
        let all_text = format!(
            "{} {} {} {}",
            self.name, self.display_name, self.description, self.vehicle
        );
        Cow::Owned(all_text)
    }

    fn preview(&self) -> ItemPreview {
        let width = textwrap::termwidth() / 2 - 1;
        ItemPreview::AnsiText(self.description(width))
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Eq for Definition {}

impl PartialEq for Definition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Ord for Definition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Definition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for User {
    fn default() -> Self {
        User::Standard
    }
}

struct Selection(i64, String);

impl Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(f)
    }
}

/// custom deserializer to parse something from a String
fn de_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// custom deserializer to parse a key from String
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
