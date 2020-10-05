use std::fmt::{self, Display, Formatter};
use std::io;

use mavlink::common::*;
use skim::{prelude::*, SkimItem};

use crate::{
    definitions::{self, Definition, User},
    mavlink_stub::MavlinkConnectionHandler,
    util::*,
};

// API

/// Represents a single parameter according to the MAVLink specification.
///
/// If the type of a variable is not `f32`, it has to be converted to `f32`.  For examle `0b1101u8`
/// becomes `13f32`.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: f32,
}

impl Parameter {
    /// Try to find a Definition for the Parameter.
    ///
    /// If not suitable Definition is found, this defaults to a sensible default.
    pub fn definition(&self) -> Definition {
        match definitions::lookup(&self.name) {
            Some(def) => def,
            None => {
                let unknown = String::from("unknown");
                Definition {
                    name: self.name.clone(),
                    description: String::from("This parameter is unknown."),
                    display_name: unknown.clone(),
                    user: User::Advanced,
                    data: None,
                    vehicle: unknown,
                }
            }
        }
    }

    /// Interact with the user to allow mutate the value.
    ///
    /// This takes over control over the terminal, and thus may disrupt other output.
    pub fn mutate(&mut self) {
        let def = self.definition();
        self.value = def.interact(self.value);
    }

    pub async fn push(&self, conn: &MavlinkConnectionHandler) -> io::Result<()> {
        let message = MavMessage::PARAM_SET(PARAM_SET_DATA {
            param_value: self.value,
            target_system: 0,
            target_component: 0,
            param_id: to_char_arr(&self.name),
            param_type: Default::default(),
        });
        conn.send_default(&message)?;
        Ok(())
    }
}

// Implementation details
impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.definition().fmt(f)
    }
}

impl SkimItem for Parameter {
    fn display(&self, _cx: skim::DisplayContext) -> AnsiString {
        AnsiString::parse(&self.definition().name())
    }

    fn text(&self) -> Cow<str> {
        let def = self.definition();
        let all_text = format!(
            "{}\n{}\n{}\n{}",
            def.name, def.display_name, def.description, def.vehicle
        );
        Cow::Owned(all_text)
    }

    fn preview(&self, _cx: skim::PreviewContext) -> ItemPreview {
        let width = textwrap::termwidth() / 2 - 1;
        ItemPreview::AnsiText(self.definition().description(width))
    }
}
