use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::models::metadata::{DependencyKind, Metadata, MetadataDependency};

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub name: String,
    pub req: semver::VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: DependencyKind,
    pub registry: Option<Url>,
    pub package: Option<String>,
}

/// The package information returned from the index as described in the Cargo reference:
/// https://doc.rust-lang.org/cargo/reference/registry-index.html
#[derive(Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<Dependency>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
    pub links: Option<String>,
}

impl PackageInfo {
    pub fn from_metadata(metadata: Metadata, checksum: &str) -> Self {
        let deps = metadata.deps.into_iter().map(Into::into).collect();
        Self {
            name: metadata.name,
            vers: metadata.vers,
            deps,
            cksum: checksum.to_string(),
            features: metadata.features,
            yanked: metadata.yanked,
            links: metadata.links,
        }
    }
}

impl From<MetadataDependency> for Dependency {
    fn from(value: MetadataDependency) -> Self {
        let (name, package) = if let Some(local_new_name) = value.explicit_name_in_toml {
            (local_new_name, value.name.into())
        } else {
            (value.name.clone(), None)
        };

        Self {
            name,
            req: value.version_req,
            features: value.features,
            optional: value.optional,
            default_features: value.default_features,
            target: value.target,
            kind: value.kind.unwrap_or(DependencyKind::Normal),
            registry: value.registry,
            package,
        }
    }
}
