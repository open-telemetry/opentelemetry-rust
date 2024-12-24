use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use xshell::{cmd, Shell};

pub mod publish;

#[derive(serde::Deserialize)]
pub struct Cargo {
    pub package: Package,
}

#[derive(serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

pub fn project_root(sh: &Shell) -> Result<String> {
    Ok(cmd!(sh, "git rev-parse --show-toplevel").read()?)
}

pub fn read_cargo_toml(p: PathBuf) -> Result<Cargo> {
    let contents = fs::read_to_string(&p)?;
    match toml::from_str(&contents) {
        Ok(d) => Ok(d),
        Err(e) => anyhow::bail!("Failed to read {}: {e}", p.to_string_lossy()),
    }
}
