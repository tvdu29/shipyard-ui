#![deny(missing_docs)]
//! The main library interface

extern crate serde;

#[cfg(feature = "default")]
extern crate actix;


#[cfg(feature = "default")]
extern crate actix_web;

#[cfg(feature = "default")]
extern crate cached;

#[cfg(feature = "default")]
mod backend;

#[cfg(feature = "default")]
pub use backend::server::Server;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate stdweb;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate yew;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use frontend::components::root::RootComponent;
use serde::Deserialize;

/// struct to parse /_catalog requests to
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Repos {
    /// list of image names
    pub repositories: Vec<String>,
}

/// struct to parse /tags requests to
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Tags {
    /// name of the image
    pub name: String,
    /// list of tags for specified image
    pub tags: Vec<String>,
}
