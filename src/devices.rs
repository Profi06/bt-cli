use std::{collections::HashMap, io::*, process::{Command, Output, Stdio}};
use colored::*;

#[derive(Debug, Clone)]
pub struct Device {
    // Name and address are the only
    // fields that should always be known
    name: String, 
    address: String,

    alias: Option<String>,
    paired: Option<bool>,
    bonded: Option<bool>,
    trusted: Option<bool>,
    blocked: Option<bool>,
    connected: Option<bool>,

    battery: Option<u8>,
}

#[derive(Debug)]
pub struct DeviceList {
    devices: Vec<Device>,

    // Following properties are saved for output
    contains_whitespaced_names: bool,
    max_name_len: u32,
}

// Used for conversion from bluetoothctl to Device
// during fill_info method
enum InfoType<'a> {
    OptBoolean(&'a mut Option<bool>),
    OptString(&'a mut Option<String>),
    OptBattery(&'a mut Option<u8>),
}

impl Device {
    pub fn new(address: &str, name: &str) -> Device {
        let name = name.to_string();
        let address = address.to_string();
        Device {
            name,
            address,
            alias: None,
            paired: None,
            bonded: None,
            trusted: None,
            blocked: None,
            connected: None,
            battery: None,
        }
    }

    pub fn update_info(&mut self) -> &mut Device {
        let cmd = Command::new("bluetoothctl").args(["info", &self.address]).output().expect("failed to execute bluetoothctl info");
        // Early return if device info was not successful
        if !cmd.status.success() {
            return self;
        }
        let mut value_hashmap = HashMap::from([
            ("Alias: ", InfoType::OptString(&mut self.alias)),
            ("Paired: ", InfoType::OptBoolean(&mut self.paired)),
            ("Bonded: ", InfoType::OptBoolean(&mut self.bonded)),
            ("Trusted: ", InfoType::OptBoolean(&mut self.trusted)),
            ("Blocked: ", InfoType::OptBoolean(&mut self.blocked)),
            ("Connected: ", InfoType::OptBoolean(&mut self.connected)),
            ("Battery Percentage: ", InfoType::OptBattery(&mut self.battery)),
        ]);
        let str = String::from_utf8(cmd.stdout).unwrap_or("".to_string());
        for line in str.lines() {
            let line = line.trim_start();
            for (text, infotype) in &mut value_hashmap {
                // Check that Line starts with specified text
                if !line.starts_with(text) {
                    continue;
                }
                // Set property
                match infotype {
                    InfoType::OptBoolean(property) => **property = Some(line.contains("yes")),
                    InfoType::OptString(property) => **property = Some(line.strip_prefix(text).expect("Prefix should exist").to_string()),
                    InfoType::OptBattery(property) => **property = line.split(&['(', ')'][..]).nth(1).and_then(|val| val.parse().ok()),
                }
            }
        }
        self
    }

    /// Attempts to pair within timeout seconds
    pub fn pair(&self, timeout: u32) -> bool {
        Command::new("bluetoothctl")
            .args(["--timeout", &timeout.to_string(), "pair", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    return out.contains("Pairing successful");
                };
                false
            })
    }

    /// Attempts to unpair
    pub fn unpair(&self) -> bool {
        Command::new("bluetoothctl")
            .args(["remove", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    return out.contains(&("[DEL] Device ".to_owned() + &self.address + " " + &self.name));
                };
                false
            })
    }

    /// Attempts to connect within timeout seconds
    pub fn connect(&self, timeout: u32) -> bool {
        Command::new("bluetoothctl")
            .args(["--timeout", &timeout.to_string(), "connect", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    return out.contains("Connection successful");
                };
                false
            })
    }

    /// Attempts to connect within timeout seconds
    pub fn disconnect(&self) -> bool {
        Command::new("bluetoothctl")
            .args(["disconnect", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    return out.contains(&("[CHG] Device ".to_owned() + &self.address + " Connected: no"));
                };
                false
            })
    }

    /// Returns the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the device name in color
    /// based on device state
    pub fn name_colored(&self) -> ColoredString {
        if self.connected == Some(true) {
            self.name.bold().green()
        } else if self.paired == Some(true) {
            self.name.to_string().into()
        } else {
            self.name.bright_black()
        }
    }

    /// Returns the device MAC Address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns true if device is paired
    pub fn is_paired(&self) -> bool {
        self.paired.unwrap_or(false)
    }

    /// Returns true if device is bonded
    pub fn is_bonded(&self) -> bool {
        self.bonded.unwrap_or(false)
    }

    /// Returns true if device is trusted
    pub fn is_trusted(&self) -> bool {
        self.trusted.unwrap_or(false)
    }

    /// Returns true if device is blocked
    pub fn is_blocked(&self) -> bool {
        self.blocked.unwrap_or(false)
    }

    /// Returns true if device is connected
    pub fn is_connected(&self) -> bool {
        self.connected.unwrap_or(false)
    }

    /// Returns the battery at the point of last update
    pub fn battery(&self) -> Option<u8> {
        self.battery
    }
}

impl DeviceList {
    pub fn new(scan_secs : Option<u32>) -> DeviceList {
        let mut devices : Vec<Device> = Vec::new();
        let mut contains_whitespaced_names = false;
        let mut max_name_len = 0;
        let mut devices_args = vec!["devices"];
        if let Some(scan_secs) = scan_secs {
            // bluetoothctl scan for unpaired nearby devices
            let _ = Command::new("bluetoothctl")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .args(["--timeout", &scan_secs.to_string(), "scan", "on"])
                .status();
        } else {
            devices_args.push("Paired");
        }
        let bluetoothctl_output : Output = Command::new("bluetoothctl")
            .args(devices_args)
            .output()
            .expect("failed to execute bluetoothctl devices");

        let output_str = String::from_utf8(bluetoothctl_output.stdout).unwrap_or("".to_string());
        for line in output_str.lines() {
            let mut split = line.splitn(3, ' ');
            // First should always be "Device" and line is therefore invalid if not
            // (for example by delayed device change notifications from scan)
            if split.next() != Some("Device") {
                continue;
            }
            if let (Some(address), Some(name)) = (split.next(), split.next()) {
                contains_whitespaced_names |= name.contains(char::is_whitespace);
                devices.push(Device::new(address, name));
            }
        }
        DeviceList { devices , contains_whitespaced_names, max_name_len }
    }

    pub fn devices_with_name(&self, name : &str) -> Vec<&Device> {
        let mut devices = Vec::new();
        for device in &self.devices {
            if device.name == name {
                devices.push(device);
            }
        }
        devices
    }

    pub fn print(&mut self, long_output: bool) {
        let mut stdout = stdout().lock();
        for device in &mut self.devices {
            device.update_info();
            if long_output {
                let _ = writeln!(stdout, "{} {}", device.address(), device.name_colored());
            } else {
                let _ = write!(stdout, "{}  ", device.name_colored());
            }
        }
        if !long_output {
            // Newline
            let _ = writeln!(stdout);
        }
    }
}

impl IntoIterator for DeviceList {
    type Item = Device;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}
