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

pub mod protocol_capnp {
    #![allow(dead_code)]
    #![allow(missing_docs)]
    #![allow(unknown_lints)]
    #![allow(clippy)]
    include!(concat!(env!("OUT_DIR"), "/src/protocol_capnp.rs"));
}
