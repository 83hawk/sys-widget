#![allow(dead_code)] // This tells Rust: "Don't worry about unused stuff here"
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub theme: String,
    pub refresh_interval: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            refresh_interval: Some(1),
        }
    }
}

pub fn load_config() -> Config {
    let path = "config.toml";

    match fs::read_to_string(path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Config parse error: {e}");
            Config::default()
        }),
        Err(_) => {
            eprintln!("Config not found, using defaults");
            Config::default()
        }
    }
}
