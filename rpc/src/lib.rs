extern crate serde;
extern crate serde_json;
extern crate xi_core_lib;

mod config;
mod annotations;
mod action;
mod mode;
mod motion;
pub mod theme;

pub use config::Config;
pub use theme::Theme;
pub use theme::Style;
pub use theme::Colour;
pub use annotations::{
    Annotation,
    AnnotationType,
};
pub use action::*;
pub use mode::*;
pub use motion::*;

