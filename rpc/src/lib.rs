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
mod view;
pub mod theme;

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
pub use view::*;

pub use xi_core_lib::rpc::*;
pub use xi_core_lib::config::ConfigDomainExternal;
pub use xi_core_lib::view::Size;
pub use xi_core_lib::LanguageId;
