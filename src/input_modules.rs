use anyhow::{Context, Result, anyhow};
use qmk_via_api::api::KeyboardApi;

use crate::{BrightnessDriver, Config};

pub struct InputModule {
    pid: ModulePID,
    qmk_api: KeyboardApi,
}

impl BrightnessDriver for InputModule {
    fn check_brightness_change(&mut self, old_brightness: u8) -> Option<u8> {
        match self.qmk_api.get_backlight_brightness() {
            Ok(b) if b == old_brightness => None,
            Ok(b) => Some(b),
            Err(err) => {
                eprintln!("failed to get input module {self} brightness: {err:#}",);
                None
            }
        }
    }

    fn set_brightness(&mut self, brightness: u8) -> u8 {
        // set the actual backlight brightness
        if let Err(err) = self.qmk_api.set_backlight_brightness(brightness) {
            eprintln!("failed to set input module {self} brightness: {err:#}");
        }

        // if this module has RGB and we should adjust the HSV value, do so
        if self.pid.has_rgb()
            && Config::get().rgb_set_hsv_value_to_brightness
            && let Err(err) = self.qmk_api.set_rgb_matrix_brightness(brightness)
        {
            eprintln!("failed to set input module {self} HSV value: {err:#}");
        }

        // our actual backlight brightness might be more coarse than an u8
        match self.qmk_api.get_backlight_brightness() {
            Ok(b) => b,
            Err(err) => {
                eprintln!("failed to get input module {self} brightness: {err:#}",);
                brightness
            }
        }
    }
}

impl std::fmt::Display for InputModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pid.fmt(f)
    }
}

// we assume the laptop is fitted with
//  - the main keyboard module
//  - one optional extra input module (numpad, macropad, ...)
pub struct InputModules {
    pub keyboard: InputModule,
    pub extra: Option<InputModule>,
}

impl InputModules {
    pub fn discover() -> Result<Self> {
        let mut keyboard = None;
        let mut extra = None;

        // scan all connected QMK / VIA devices
        for dev in qmk_via_api::scan::scan_keyboards() {
            // check if this is a Framework Laptop 16 input module
            if dev.vendor_id != FRAMEWORK_VID {
                continue;
            }

            let Ok(pid) = ModulePID::try_from(dev.product_id) else {
                continue;
            };

            // open the input module's QMK/VIA device
            let qmk_api =
                KeyboardApi::new(FRAMEWORK_VID, pid as u16, dev.usage_page).map_err(|e| {
                    anyhow!("{e}")
                        .context(format!("failed to open input module {pid} QMK/VIA device"))
                })?;

            let slot = match pid.kind() {
                ModuleKind::Keyboard => &mut keyboard,
                ModuleKind::Numpad | ModuleKind::Macropad => &mut extra,
            };

            if slot.is_none() {
                *slot = Some(InputModule { pid, qmk_api });
            } else {
                eprintln!("ignoring extra input module {pid}");
            }
        }

        Ok(Self {
            keyboard: keyboard.context("no keyboard input module found")?,
            extra,
        })
    }
}

const FRAMEWORK_VID: u16 = 0x32ac;

#[repr(u16)]
#[derive(Debug, Clone, Copy, num_enum::TryFromPrimitive)]
enum ModulePID {
    KeyboardANSI = 0x0012,
    KeyboardANSICopilot = 0x0030,
    KeyboardISO = 0x0018,
    KeyboardJIS = 0x0019,
    Numpad = 0x0014,
    Macropad = 0x0013,
}

#[derive(Debug, Clone)]
enum ModuleKind {
    Keyboard,
    Numpad,
    Macropad,
}

impl ModulePID {
    const fn kind(self) -> ModuleKind {
        match self {
            ModulePID::KeyboardANSI
            | ModulePID::KeyboardANSICopilot
            | ModulePID::KeyboardISO
            | ModulePID::KeyboardJIS => ModuleKind::Keyboard,
            ModulePID::Numpad => ModuleKind::Numpad,
            ModulePID::Macropad => ModuleKind::Macropad,
        }
    }

    const fn has_rgb(self) -> bool {
        match self {
            ModulePID::KeyboardANSI => {
                // the RGB and the pure-white backlight variants have the same
                // PID, which means we can't tell them apart (and there's no VIA
                // API to tell either)
                true
            }
            ModulePID::Macropad => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for ModulePID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)?;
        f.write_fmt(format_args!(" ({FRAMEWORK_VID:04x}:{:04x})", *self as u16))?;
        Ok(())
    }
}
