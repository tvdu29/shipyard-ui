#![deny(missing_docs)]
//! The main library interface

extern crate serde;
extern crate serde_json;

#[cfg(feature = "default")]
extern crate actix;


#[cfg(feature = "default")]
extern crate actix_web;

#[cfg(feature = "default")]
extern crate cached;

#[cfg(feature = "default")]
mod backend;

use anyhow::Error;
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
use serde::{Deserialize, Serialize};
use crate::DockerManifest::*;

///docker manifest parser
pub fn get_manifest(manifest: &str) -> Result<DockerManifest, anyhow::Error> {
    let config: SchemaVersion = serde_json::from_str(manifest).expect("Failed to parse manifest");
    match (config.schema_version, config.media_type) {
        (1, _) => Ok(V1(serde_json::from_str(manifest).expect("Failed to parse manifest"))),
        (2, Some(media_type)) => match media_type.as_str() {
            "application/vnd.docker.distribution.manifest.list.v2+json" => Ok(V2List(serde_json::from_str(manifest).expect("Failed to parse manifest"))),
            "application/vnd.docker.distribution.manifest.v2+json" => Ok(V2(serde_json::from_str(manifest).expect("Failed to parse manifest"))),
            _ => Err(Error::msg("Invalid media type"))
        },
        (_, _) => Err(Error::msg("Invalid schema version")),
    }
}

/// struct to parse `/catalog` requests to
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Repos {
    /// list of image names
    pub repositories: Vec<String>,
}

/// struct to parse `/tags` requests to
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Tags {
    /// name of the image
    pub name: String,
    /// list of tags for specified image
    pub tags: Vec<String>,
}

///enum for Docker manifest version
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DockerManifest {
    ///schema version: 1
    V1(Manifest),
    ///schema version: 2
    V2(ManifestV2),
    ///schema version: 2 + media_type: application/vnd.docker.distribution.manifest.list.v2+json
    V2List(ManifestV2List),
}

///struct for deserializing schema version
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersion {
    schema_version: usize,
    media_type: Option<String>,
    errors: Option<ErrorsV2>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
///struct for manifest v2 config
pub struct ManifestV2ListPlatform {
    architecture: String,
    os: String,
    variant: Option<String>,
    features: Option<Vec<String>>
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
///struct for manifest v2 config
pub struct ManifestConfig {
    media_type: String,
    size: usize,
    digest: String,
    platform: Option<ManifestV2ListPlatform>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
///struct for parsing error details from registry
pub struct ErrorsV2Detail{
    tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
///struct for parsing errors from registry
pub struct ErrorsV2{
    code: String,
    message: String,
    detail: ErrorsV2Detail,
}

/// struct to parse `/manifest` v1 requests to
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Manifest{
    name: String,
    tag: String,
    architecture: String,
    fs_layers: Vec<String>,
    history: Vec<String>
}

/// struct to parse `/manifest` v2 requests to
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV2{
    media_type: Option<String>,
    manifests: Option<ManifestConfig>,
    layers: Option<Vec<ManifestConfig>>,
}

/// struct to parse `/manifest` v2 requests to
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV2List{
    media_type: Option<String>,
    config: Option<ManifestConfig>,
    layers: Option<Vec<ManifestConfig>>,
    errors: Option<ErrorsV2>
}