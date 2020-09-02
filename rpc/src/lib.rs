extern crate serde;
extern crate serde_json;
extern crate xi_core_lib;

mod annotations;
mod action;
mod config;
mod quantity;
mod mode;
mod motion;
mod find;
mod plugins;
pub mod theme;

pub use xi_core_lib::rpc::GestureType;
pub use xi_core_lib::rpc::SelectionGranularity;

pub use config::Config;
pub use theme::Theme;
pub use theme::Style;
pub use theme::Colour;
pub use annotations::{
    Annotation,
    AnnotationType,
};
pub use find::*;
pub use action::*;
pub use quantity::*;
pub use mode::*;
pub use motion::*;
pub use plugins::*;

