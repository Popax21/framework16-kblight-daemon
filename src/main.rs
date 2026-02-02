use anyhow::{Context, Result};

mod config;
mod input_modules;
mod uleds;

use crate::uleds::UserspaceLED;
pub use crate::{config::Config, input_modules::InputModules};

fn main() -> Result<()> {
    Config::load()?;

    // discover input modules
    let InputModules { keyboard, extra } =
        InputModules::discover().context("failed to discover input modules")?;

    if let Some(extra) = &extra {
        println!("discovered keyboard input module {keyboard} with extra module {extra}",);
    } else {
        println!("discovered keyboard input module {keyboard} with no extra modules",);
    }

    // build a list of "brightness drivers", each of which can
    //  - trigger brightness change events
    //  - change its current brightness
    let mut drivers = Vec::new();

    drivers.push(DriverState::new(keyboard, "keyboard input module"));

    if let Some(extra) = extra {
        drivers.push(DriverState::new(extra, "extra input module"));
    }

    // create a new uleds LED if enabled, which is a "brightness driver" backed
    // by the kernel LED interface, which is in turn driven by e.g. the
    // brightness slider in desktop environments
    const ULED_NAME: &str = "fw16::kbd_backlight";
    if Config::get().expose_via_uleds {
        match UserspaceLED::new(ULED_NAME.to_owned()) {
            Ok(uled) => {
                println!("registered {ULED_NAME:?} uleds kernel LED");
                drivers.push(DriverState::new(uled, "kernel LED"));
            }
            Err(err) => {
                eprintln!(
                    "failed to expose uleds LED (is the `uleds` kernel module loaded?): {err:#}"
                );
            }
        }
    }

    // poll all drivers for brightness changes
    let poll_interval = std::time::Duration::from_millis(Config::get().poll_interval_ms);

    'poll: loop {
        let new_brightness = 'check: {
            for driver in &mut drivers {
                if let Some(new_brightness) = driver.check_brightness_change() {
                    if Config::get().verbose {
                        println!(
                            "keyboard backlight brightness change (source: {}) -> {:.0}%",
                            driver.name,
                            new_brightness as f64 / 255.0 * 100.0,
                        );
                    }

                    break 'check new_brightness;
                }
            }

            // no new brightness, wait a bit then loop
            std::thread::sleep(poll_interval);
            continue 'poll;
        };

        // update the brightness of all drivers
        for driver in &mut drivers {
            driver.set_brightness(new_brightness);
        }
    }
}

struct DriverState {
    name: &'static str,
    driver: Box<dyn BrightnessDriver>,
    // we track the current brightness per-driver since e.g. the input modules
    // are way more coarse than e.g the uled, which would otherwise lead to
    // spurious change events
    cur_brightness: u8,
}

impl DriverState {
    fn new(driver: impl BrightnessDriver + 'static, name: &'static str) -> Self {
        Self {
            name,
            driver: Box::new(driver),
            cur_brightness: 0,
        }
    }

    fn check_brightness_change(&mut self) -> Option<u8> {
        self.driver.check_brightness_change(self.cur_brightness)
    }

    fn set_brightness(&mut self, brightness: u8) {
        // the driver returns the actual brightness that was set
        self.cur_brightness = self.driver.set_brightness(brightness);
    }
}

pub trait BrightnessDriver {
    fn check_brightness_change(&mut self, old_brightness: u8) -> Option<u8>;
    fn set_brightness(&mut self, brightness: u8) -> u8;
}
