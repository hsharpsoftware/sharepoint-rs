#[macro_use]
extern crate hyper;

#[macro_use]
extern crate serde_derive;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

mod data;

pub mod auth;
pub mod list;

#[derive(Debug, Clone)]
pub struct Site {
    pub parent: String,
}
