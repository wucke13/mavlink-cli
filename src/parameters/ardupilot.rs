use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use serde::Deserialize;
use serde_json::from_str;

use super::*;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ArduPilotDefinitions {
    #[serde(rename = "json")]
    _json: Meta,

    #[serde(flatten)]
    pub(super) vehicles: HashMap<String, HashMap<String, Definition>>,
}

#[derive(Debug, Clone, Deserialize)]
struct Meta {
    version: i64,
}

pub(super) fn parse(input: &str) -> io::Result<HashMap<String, Definition>> {
    let def: ArduPilotDefinitions =
        from_str(input).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut map = HashMap::new();

    for (vehicle, mut param_map) in def.vehicles {
        for (param_name, mut param) in param_map {
            param.vehicle = vehicle.clone();
            param.name = param_name.clone();
            map.insert(param_name, param);
        }
    }
    Ok(map)
}
