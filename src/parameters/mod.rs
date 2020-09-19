use std::fmt::{self, Display, Formatter};

use skim::{prelude::*, SkimItem};

mod ardupilot;
pub mod definitions;

use definitions::{Definition, User};

// API

/// Represents a single parameter according to the MAVLink specification.
///
/// If the type of a variable is not `f32`, it has to be converted to `f32`.  For examle `0b1101u8`
/// becomes `13f32`.
#[derive(Debug, Clone)]
pub struct Parameter {
    name: String,
    value: f32,
}

impl Parameter {
    /// Try to find a Definition for the Parameter.
    ///
    /// If not suitable Definition is found, this defaults to a sensible default.
    pub fn definition(&self) -> Definition {
        match definitions::lookup(&self.name) {
            Some(def) => def.clone(),
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
}

// Implementation details
impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.definition().fmt(f)
    }
}

impl SkimItem for Parameter {
    fn display(&self) -> Cow<AnsiString> {
        Cow::Owned(AnsiString::parse(&self.definition().name()))
    }

    fn text(&self) -> Cow<str> {
        let def = self.definition();
        let all_text = format!(
            "{}\n{}\n{}\n{}",
            def.name, def.display_name, def.description, def.vehicle
        );
        Cow::Owned(all_text)
    }

    fn preview(&self) -> ItemPreview {
        let width = textwrap::termwidth() / 2 - 1;
        ItemPreview::AnsiText(self.definition().description(width))
    }
}
