use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::BrightnessDriver;

pub struct UserspaceLED {
    file: File,
    sysfs_brightness_path: PathBuf,
}

impl UserspaceLED {
    pub fn new(name: String) -> Result<Self> {
        assert!(name.len() <= 64);

        // open /dev/uleds and write a uleds_user_dev struct to it
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uleds")
            .context("failed to open /dev/uleds")?;

        // struct uleds_user_dev {
        //     char name[64];
        //     int max_brightness;
        // };

        let mut user_dev = [0u8; 64 + 4];
        user_dev[..name.len()].copy_from_slice(name.as_bytes());
        user_dev[64..].copy_from_slice(&255u32.to_ne_bytes());

        assert_eq!(
            file.write(&user_dev)
                .context("failed to write struct uleds_user_dev")?,
            user_dev.len(),
        );

        // construct the uled instance
        let mut sysfs_brightness_path = PathBuf::from("/sys/class/leds");
        sysfs_brightness_path.push(name);
        sysfs_brightness_path.push("brightness");

        let mut uled = Self {
            file,
            sysfs_brightness_path,
        };

        // swallow the first "brightness change"
        assert!(uled.check_brightness_change(0).is_none());

        use nix::fcntl::*;
        fcntl(&uled.file, F_SETFL(OFlag::O_NONBLOCK)).context("failed to set O_NONBLOCK")?;

        Ok(uled)
    }
}

impl BrightnessDriver for UserspaceLED {
    fn check_brightness_change(&mut self, old_brightness: u8) -> Option<u8> {
        let mut data: [u8; 4] = [0u8; 4];
        match self.file.read(&mut data) {
            Ok(4) => {
                let brightness = u32::from_ne_bytes(data) as u8;
                (brightness != old_brightness).then_some(brightness)
            }
            Ok(_) => unreachable!("uleds didn't read all 4 bytes"),
            Err(err) if err.kind() == ErrorKind::WouldBlock => None,
            Err(err) => {
                eprintln!("failed to read uleds brightness change: {err:#}");
                None
            }
        }
    }

    fn set_brightness(&mut self, brightness: u8) -> u8 {
        if let Err(err) = std::fs::write(&self.sysfs_brightness_path, brightness.to_string()) {
            eprintln!("failed to update uleds sysfs brightness: {err:#}");
        }
        brightness
    }
}
