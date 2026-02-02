use std::{path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The polling interval used to check for backlight brightness changes.
    pub poll_interval_ms: u64,
    /// Set the HSV value of RGB-capable input modules to the backlight
    /// brightness, mimicking the behavior of the default backlight brightness
    /// step keys.
    pub rgb_set_hsv_value_to_brightness: bool,
    /// Expose the keyboard's brightness via the uleds kernel module to allow
    /// for control using the Linux LED subsystem and its user (like desktop
    /// environments).
    pub expose_via_uleds: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval_ms: 200,
            rgb_set_hsv_value_to_brightness: Default::default(),
            expose_via_uleds: true,
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn load() -> Result<()> {
        let config_path: PathBuf = std::env::var("CONFIG_FILE")
            .ok()
            .unwrap_or("/etc/fw16-kblight.toml".to_owned())
            .into();

        let config = if config_path.exists() {
            println!("reading config file from {config_path:?}");

            let config_data =
                std::fs::read_to_string(&config_path).context("failed to read config file")?;
            toml::from_str(&config_data).context("failed to parse config file")?
        } else {
            println!("config file {config_path:?} does not exist, using defaults");
            Self::default()
        };

        CONFIG.set(config).ok().context("config already loaded")
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("no config loaded yet")
    }
}
