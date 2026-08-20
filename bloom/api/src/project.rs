use semver::Version;
use serde::Deserialize;

use crate::error::BloomError;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub bloom: BloomConfig,
    pub dev: DevConfig,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BloomConfig {
    pub version: Version,
    pub min_client_version: Version,
}

#[derive(Debug, Deserialize)]
pub struct DevConfig {
    pub frontend: DevProcess,
    pub client: DevProcess,
}

#[derive(Debug, Deserialize)]
pub struct DevProcess {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub watch: Vec<String>,
}

impl ProjectConfig {
    pub fn load() -> Result<Self, BloomError> {
        let contents =
            std::fs::read_to_string("proj.bloom.yml").map_err(|_| BloomError::ProjectNotFound)?;
        let config: ProjectConfig =
            yaml_serde::from_str(&contents).map_err(|_| BloomError::InvalidProject)?;

        Ok(config)
    }
}
