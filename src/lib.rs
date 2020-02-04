#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate async_trait;
extern crate config as config_rs;

pub mod config;
pub mod error;
pub mod issue_tracker;
pub mod parser;

pub type Result<T> = std::result::Result<T, error::Error>;
