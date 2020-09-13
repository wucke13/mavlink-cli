use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

//use xml::reader::{EventReader, XmlEvent};
use serde::Deserialize;
use serde_xml_rs::{from_reader, to_string};

pub struct Parameter {
    pub name: String,
    pub human_name: String,
    pub documentation: String,
    pub user: User,
    pub data_type: DataType,
    pub source: Source,
}

pub enum DataType {
    MultiSelect(HashMap<String, f32>),
    Select(HashMap<String, f32>),
    Range {
        from: f32,
        to: f32,
        unit: String,
        unit_text: String,
        increment: Option<f32>,
    },
}

pub enum Source {
    Unknown,
}

pub fn parse(path: &Path) -> io::Result<HashMap<String, Parameter>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let xml: ParamFile =
        from_reader(file).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut result = HashMap::new();

    for vehicle in xml.vehicles {
        for param in vehicle.parameters.param {
            let data_type = match (param.field, param.values) {
                // range
                (field, None) => {
                    let mut range = field
                        .iter()
                        .find(|&f| f.name == "Range")
                        .unwrap()
                        .value
                        .split_whitespace();
                    let from = range.next().unwrap().parse().unwrap();
                    let to = range.next().unwrap().parse().unwrap();
                    let increment = field
                        .iter()
                        .find(|&f| f.name == "Increment")
                        .unwrap()
                        .value
                        .parse()
                        .ok();
                    let unit = field
                        .iter()
                        .find(|&f| f.name == "Units")
                        .unwrap()
                        .value
                        .clone();
                    let unit_text = field
                        .iter()
                        .find(|&f| f.name == "UnitText")
                        .unwrap()
                        .value
                        .clone();

                    DataType::Range {
                        from,
                        to,
                        unit,
                        unit_text,
                        increment,
                    }
                }
                (field, Some(values)) if field.is_empty() => DataType::Select(
                    values
                        .value
                        .into_iter()
                        .map(|v| (v.meaning, parse_dirty_float(&v.code).unwrap()))
                        .collect(),
                ),
                (field, Some(values)) => DataType::MultiSelect(
                    field
                        .into_iter()
                        .find(|f| f.name == "Bitmask")
                        .unwrap()
                        .value
                        .split(',')
                        .map(|bit| {
                            let mut s = bit.split(':');
                            let v = s.next().unwrap().parse().unwrap();
                            let k = s.next().unwrap();
                            (String::from(k),v)
                        }).collect(),
                ),
            };

            result.insert(
                param.name.clone(),
                Parameter {
                    name: param.name,
                    human_name: param.human_name,
                    documentation: param.documentation,
                    user: param.user,
                    data_type: data_type,
                    source: Source::Unknown,
                },
            );
        }
    }

    //panic!("{:#?}", xml);

    Ok(result)
}

/// Parses a string into a float, even if there is some whitespace arround the digits.
fn parse_dirty_float(s: &str)->io::Result<f32>{
    let sanitized_string: String =   s.chars().filter(|c| !c.is_whitespace()).collect();
    sanitized_string.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[derive(Debug, Clone, Deserialize)]
struct ParamFile {
    vehicles: Vec<Vehicle>,
    libraries: Vec<Library>,
}

#[derive(Debug, Clone, Deserialize)]
struct Library {
    parameters: Vec<Parameters>,
}

#[derive(Debug, Clone, Deserialize)]
struct Vehicle {
    parameters: Parameters,
}

#[derive(Debug, Clone, Deserialize)]
struct Parameters {
    name: String,
    param: Vec<Param>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Param {
    human_name: String,
    name: String,
    documentation: String,
    #[serde(default)]
    user: User,
    values: Option<Values>,
    #[serde(default,rename = "field")]
    field: Vec<Field>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct Field {
    name: String,
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum User {
    Standard,
    Advanced,
}

#[derive(Debug, Clone, Deserialize)]
struct Values {
    value: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct Value {
    code: String,
    #[serde(rename = "$value")]
    meaning: String,
}

impl Default for User {
    fn default() -> Self {
        User::Standard
    }
}
