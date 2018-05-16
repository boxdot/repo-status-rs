use serde_xml_rs::deserialize;

use failure::Error;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub mod project;

#[derive(Deserialize, Debug)]
pub struct Manifest {
    #[serde(rename = "remote", default)]
    pub remotes: Vec<Remote>,
    #[serde(rename = "default", default)]
    pub defaults: Vec<Default>,
    #[serde(rename = "project", default)]
    pub projects: Vec<project::Project>,
}

#[derive(Debug, Deserialize)]
pub struct Default {
    pub revision: String,
    pub remote: String,
}

#[derive(Debug, Deserialize)]
pub struct Remote {
    pub name: String,
    pub fetch: String,
    pub review: String,
}

#[derive(Debug, Fail)]
enum ManifestError {
    #[fail(display = "manifest does not exists at: {}", path)]
    ManifestDoesNotExists { path: String },
}

impl Manifest {
    pub fn from_current_dir() -> Result<Manifest, Error> {
        Manifest::from_path(&env::current_dir()?)
    }

    fn from_path(path: &Path) -> Result<Manifest, Error> {
        let manifest_path = path.join(".repo/manifest.xml");
        if !manifest_path.exists() {
            return Err(ManifestError::ManifestDoesNotExists {
                path: String::from(path.to_string_lossy()),
            }.into());
        }
        let file = File::open(manifest_path)?;
        let reader = BufReader::new(file);
        let manifest: Manifest = deserialize(reader)?;
        Ok(manifest)
    }
}
