use anyhow::{Context, Result};

mod config;
mod input_modules;
mod uleds;

use crate::uleds::UserspaceLED;
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

    // create a new uleds LED if enabled
    const ULED_NAME: &str = "fw16::kbd_backlight";
    let mut uled = if Config::get().expose_via_uleds {
        match UserspaceLED::new(ULED_NAME.to_owned()) {
            Ok(uled) => {
                println!("exposing {ULED_NAME:?} uleds kernel LED");
                Some(uled)
            }
            Err(err) => {
                eprintln!(
                    "failed to expose uleds LED (is the `uleds` kernel module loaded?): {err:#}"
                );
                None
            }
        }
    } else {
        None
    };

    // poll for brightness changes
    let poll_interval = std::time::Duration::from_millis(Config::get().poll_interval_ms);

    let mut brightness = 0;
    'poll: loop {
        brightness = 'check: {
            macro_rules! check {
                ($change:expr, $source:literal) => {
                    if let Some(new_brightness) = $change.check_brightness_change(brightness) {
                        println!(
                            "keyboard backlight brightness change (source: {}): {:.0}% -> {:.0}%",
                            $source,
                            brightness as f64 / 255.0 * 100.0,
                            new_brightness as f64 / 255.0 * 100.0,
                        );

                        break 'check new_brightness;
                    }
                };
            }

            // check the kernel LED
            if let Some(uled) = &mut uled {
                check!(uled, "kernel LED");
            }

            // check input modules
            check!(input_modules.keyboard, "keyboard input module");

            if let Some(extra_module) = &input_modules.extra {
                check!(extra_module, "extra input module");
            }

            // no new brightness, loop
            std::thread::sleep(poll_interval);
            continue 'poll;
        };

        // update the keyboard backlight brightness
        input_modules.keyboard.set_brightness(brightness);

        if let Some(extra_module) = &input_modules.extra {
            extra_module.set_brightness(brightness);
        }

        if let Some(uled) = &mut uled {
            uled.set_brightness(brightness);
        }
    }
}
