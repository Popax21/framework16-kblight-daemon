use anyhow::{Context, Result};

mod config;
mod input_modules;

pub use crate::{config::Config, input_modules::InputModules};

fn main() -> Result<()> {
    Config::load()?;

    // discover input modules
    let input_modules = InputModules::discover().context("failed to discover input modules")?;

    if let Some(extra) = &input_modules.extra {
        println!(
            "discovered keyboard input module {} with extra module {extra}",
            input_modules.keyboard,
        );
    } else {
        println!(
            "discovered keyboard input module {} with no extra modules",
            input_modules.keyboard,
        );
    }

    // poll for brightness changes
    let poll_interval = std::time::Duration::from_millis(Config::get().poll_interval_ms);

    let mut brightness = 0;
    loop {
        let Some(new_brightness) = input_modules.check_brightness_change(brightness) else {
            std::thread::sleep(poll_interval);
            continue;
        };

        println!(
            "keyboard backlight brightness change: {:.0}% -> {:.0}%",
            brightness as f64 / 255.0 * 100.0,
            new_brightness as f64 / 255.0 * 100.0,
        );

        input_modules.set_brightness(new_brightness);
        brightness = new_brightness;
    }
}
