use std::{collections::HashMap, io::*, process::{Command, Output, Stdio}};
use colored::*;
use termsize::{self, Size};

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
    icon: Option<String>,
}

// Used for conversion from bluetoothctl to Device
// during fill_info method
enum InfoTypeMut<'a> {
    OptBoolean(&'a mut Option<bool>),
    OptString(&'a mut Option<String>),
    OptBattery(&'a mut Option<u8>),
}

enum InfoType<'a> {
    OptBoolean(&'a Option<bool>),
    OptString(&'a Option<String>),
    OptBattery(&'a Option<u8>),
}

impl Device {
    pub fn new(address: &str, name: &str) -> Device {
        Device {
            name: name.to_string(),
            address: address.to_string(),
            alias: None,
            paired: None,
            bonded: None,
            trusted: None,
            blocked: None,
            connected: None,
            battery: None,
            icon: None,
        }
    }

    pub fn update_info(&mut self) -> &mut Device {
        let cmd = Command::new("bluetoothctl")
            .args(["info", &self.address])
            .output().expect("failed to execute bluetoothctl info");
        // Early return if device info was not successful
        if !cmd.status.success() {
            return self;
        }
        let mut value_hashmap = HashMap::from([
            ("Alias: ", InfoTypeMut::OptString(&mut self.alias)),
            ("Paired: ", InfoTypeMut::OptBoolean(&mut self.paired)),
            ("Bonded: ", InfoTypeMut::OptBoolean(&mut self.bonded)),
            ("Trusted: ", InfoTypeMut::OptBoolean(&mut self.trusted)),
            ("Blocked: ", InfoTypeMut::OptBoolean(&mut self.blocked)),
            ("Connected: ", InfoTypeMut::OptBoolean(&mut self.connected)),
            ("Battery Percentage: ", InfoTypeMut::OptBattery(&mut self.battery)),
            ("Icon: ", InfoTypeMut::OptString(&mut self.icon)),
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
                    InfoTypeMut::OptBoolean(property) => 
                    **property = Some(line.contains("yes")),

                    InfoTypeMut::OptString(property) => 
                    **property = Some(line
                        .strip_prefix(text).expect("Prefix should exist")
                        .to_string()
                    ),

                    InfoTypeMut::OptBattery(property) => 
                    **property = line
                        // Split at left and right braces 
                        // (only included in input right before and
                        // after human readable battery percentage)
                        .split(&['(', ')'][..])
                        .nth(1).and_then(|val| 
                            val.parse().ok()
                        ),
                }
            }
        }
        self
    }

    /// Attempts to pair 
    pub fn pair(&mut self) -> bool {
        println!("Attempting to pair with {}...", self.name_colored());
        pairable(true);
        let ret = Command::new("bluetoothctl")
            .args(["pair", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    // Also consider pairing successful if it failed due to already being paired 
                    let success = out.contains("Pairing successful") || String::from_utf8(output.stderr)
                        .is_ok_and(|stderr| stderr.contains("org.bluez.Error.AlreadyExists"));
                    if success {
                        self.paired = Some(true);
                        println!("{} paired.", self.name_colored());
                    } else {
                        eprintln!("Could not pair {}.", self.name_colored());
                    }
                    return success;
                };
                false
            });
        pairable(false);
        ret
    }

    /// Unpairs the device
    pub fn unpair(&mut self) -> bool {
        println!("Attempting to remove {}...", self.name_colored());
        Command::new("bluetoothctl")
            .args(["remove", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    let success = out.contains("Device has been removed");
                    if success {
                        self.paired = Some(false);
                        self.connected = Some(false);
                        println!("{} removed.", self.name_colored());
                    } else {
                        println!("Could not remove {}.", self.name_colored());
                    }
                    return success;
                };
                false
            })
    }

    /// Attempts to trust 
    pub fn trust(&mut self) -> bool {
        Command::new("bluetoothctl")
            .args(["trust", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    // Also consider pairing successful if it failed due to already being paired 
                    let success = out.contains(&("Changing".to_owned() + &self.address + " trust succeeded"));
                    if success {
                        self.trusted = Some(true);
                    }
                    return success;
                };
                false
            })
    }

    /// Untrusts the device
    pub fn untrust(&mut self) -> bool {
        Command::new("bluetoothctl")
            .args(["remove", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    println!("{out}");
                    let success = out.contains(&("Changing".to_owned() + &self.address + " untrust succeeded"));
                    if success {
                        self.trusted = Some(false);
                    }
                    return success;
                };
                false
            })
    }

    /// Attempts to connect to device
    pub fn connect(&mut self) -> bool {
        println!("Attempting to connect with {}...", self.name_colored());
        Command::new("bluetoothctl")
            .args(["connect", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    let success = out.contains("Connection successful");
                    if success {
                        self.connected = Some(true);
                        println!("{} connected.", self.name_colored());
                    }
                    return success;
                };
                false
            })
    }

    /// Attempts to disconnect from device
    pub fn disconnect(&mut self) -> bool {
        println!("Attempting to disconnect from {}...", self.name_colored());
        Command::new("bluetoothctl")
            .args(["disconnect", &self.address])
            .output().is_ok_and(|output| {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    let success = out.contains(&("[CHG] Device ".to_owned() + &self.address + " Connected: no"));
                    if success {
                        self.connected = Some(false);
                        println!("{} disconnected.", self.name_colored());
                    }
                    return success;
                };
                false
            })
    }

    /// Colors str based on device state. 
    /// Used by some *_colored methods.
    fn to_colored(&self, str: &str) -> ColoredString {
        let mut return_value : ColoredString = str.into();
        if self.paired != Some(true) {
            return_value = return_value.bright_black();
        }
        if self.connected == Some(true) {
            return_value = return_value.bold().blue();
        } 
        return_value
    }

    /// Returns the device name with formatting.
    pub fn name_colored(&self) -> ColoredString {
        self.to_colored(&self.name)
    }

    /// normal output of name_colored with  if name
    /// contains whitespace, otherwise placeholder is added
    pub fn quoted_name_colored(&self, quotes: &str, placeholder: &str) -> ColoredString {
        let added = if self.name.contains(char::is_whitespace) { quotes } else { placeholder };
        self.to_colored(&(added.to_owned() + &self.name + added))
    }

    pub fn info_colored(&self) -> ColoredString {
        let mut return_value = format!("{} {}", self.address, self.name_colored());
        let print_props = Vec::from([
            ("\n\tAlias: ", InfoType::OptString(&self.alias)),
            ("\n\tIcon: ", InfoType::OptString(&self.icon)),
            ("\n\tConnected: ", InfoType::OptBoolean(&self.connected)),
            ("\n\tPaired: ", InfoType::OptBoolean(&self.paired)),
            ("\n\tBonded: ", InfoType::OptBoolean(&self.bonded)),
            ("\n\tTrusted: ", InfoType::OptBoolean(&self.trusted)),
            ("\n\tBlocked: ", InfoType::OptBoolean(&self.blocked)),
            ("\n\tBattery Percentage: ", InfoType::OptBattery(&self.battery)),
        ]);
        for (prefix, property) in print_props {
            match property {
                InfoType::OptString(Some(propval)) => return_value = return_value + prefix + propval,
                InfoType::OptBoolean(Some(propval)) => return_value = return_value + &format!("{prefix}{}", if *propval {"yes".green()} else {"no".red()}),
                InfoType::OptBattery(Some(percentage)) => return_value = return_value + prefix + &format!("{}", match percentage {
                    70.. => percentage.to_string().green(),
                    30.. => percentage.to_string().yellow(),
                    _ => percentage.to_string().red(),
                }),
                _ => (),
            }
        }
        return_value.into()
    }

    // Returns the length of the device name (as an u8 because
    // the bluetooth specification limits name length to 248.
    // See Section 6.23: https://www.bluetooth.com/specifications/core54-html/)
    pub fn name_len(&self) -> u8 {
        // len should match amount of characters because of limitation to UTF-8
        self.name.len().try_into().expect("Name length should adhere to bluetooth specification")
    }
}

#[derive(Debug)]
pub struct DeviceList {
    devices: Vec<Device>,

    // Following properties are saved for output
    contains_whitespaced_names: bool,
    max_name_len: u8,
    min_name_len: u8,
}

impl DeviceList {
    pub fn new(scan_secs : Option<u32>) -> DeviceList {
        let mut devices : Vec<Device> = Vec::new();
        let mut contains_whitespaced_names = false;
        let mut max_name_len = 0;
        let mut min_name_len = u8::MAX;
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
                let device = Device::new(address, name);
                max_name_len = max_name_len.max(device.name_len());
                min_name_len = max_name_len.min(device.name_len());
                devices.push(device);
            }
        }
        DeviceList { devices , contains_whitespaced_names, max_name_len, min_name_len }
    }

    /// Returns devices in device list with given name
    pub fn devices_with_name(&mut self, name : &str) -> Vec<&mut Device> {
        let mut devices = Vec::new();
        for device in &mut self.devices {
            if device.name == name {
                devices.push(device);
            }
        }
        devices
    }

    pub fn print(&mut self, linewise: bool, long_output: bool) {
        if !linewise && !long_output {
            self.print_columns();
        } else if linewise {
            self.print_line()
        } else if long_output {
            let mut stdout = stdout().lock();
            for device in &mut self.devices {
                device.update_info();
                let _ = writeln!(stdout, "{} {}", &device.address,
                    if self.contains_whitespaced_names {
                        device.quoted_name_colored("'", " ")
                    } else {
                        device.name_colored()
                    });
            }
        }
    }

    pub fn print_line(&mut self) {
        let mut stdout = stdout().lock();
        for device in &mut self.devices {
            device.update_info();
            let _ = writeln!(stdout, "{}", 
                if self.contains_whitespaced_names {
                    device.quoted_name_colored("'", " ")
                } else {
                    device.name_colored()
                });
        }
    }

    pub fn print_columns(&mut self) {
        // First find highest amount of possible colums and the best fit column widths
        #[derive(Debug)]
        struct ColsInfo { widths: Vec<u8>, total_w: u16 }
        let max_w: u16 = match termsize::get() {
            // cols is terminal width in chars
            Some(Size {rows: _, cols}) => cols,
            _ => 80,
        }.try_into().unwrap_or(80);
        // Checked div prevents divide by zero for empty names
        // Lower bound: Assume all names as long as longest
        let min_cols = max_w.checked_div(self.max_name_len.into()).unwrap_or(0);
        // Upper bound: Assume all names as long as shortest
        let max_cols = max_w.checked_div(self.min_name_len.into()).unwrap_or(max_w).min(self.devices.len().try_into().unwrap_or(max_w));
        // Fallback to print_line if max_name_len > max_w
        // or max_name_len == 0 for the sake of simplicity
        if min_cols == 0 {
            self.print_line();
            return;
        }

        // If there are whitespaced names, also account for space used by qoutes
        let extra_char_num = 2 + 2 * u8::from(self.contains_whitespaced_names);
        // Infos for every column amount considered
        let mut col_infos: Vec<ColsInfo> = Vec::new();
        col_infos.reserve((max_cols + 1 - min_cols).try_into().unwrap_or(0));
        for cols_num in min_cols..=max_cols {
            col_infos.push(ColsInfo { 
                widths: vec![0; cols_num.into()], 
                total_w: (u16::from(extra_char_num) * cols_num).try_into().unwrap_or(0),
            })
        };

        for (idx, device) in self.devices.iter().enumerate() {
            for (add_cols, col_info) in col_infos.iter_mut().enumerate() {
                // This amount of device columns has already been proven 
                // unusable. Skip to next column amount option
                if col_info.total_w > max_w {
                    break;
                }

                // Calculate column device would be displayed in 
                // add_cols + min_cols is amount of columns
                let idx = idx % (add_cols + usize::from(min_cols));
                if col_info.widths[idx] < device.name_len() {
                    let size_incr = device.name_len() - col_info.widths[idx];
                    col_info.widths[idx] += size_incr;
                    col_info.total_w += u16::from(size_incr);
                }
            }
        }
        
        // Find highest amount of columns with valid display width
        let mut col_info = &ColsInfo { widths: vec![self.max_name_len], total_w: max_w };
        for candidate in col_infos.iter().rev() {
            if candidate.total_w <= max_w {
                col_info = candidate;
                break;
            };
        };
        // Finally, print
        eprintln!("{:?}", col_info);
        let mut stdout = stdout().lock();
        for (idx, device) in self.devices.iter_mut().enumerate() {
            // Output newline when idx 0 is reached (except for first line, where newline is
            // assumed to already be present)
            if idx != 0 && idx % col_info.widths.len() == 0 {
                let _ = writeln!(stdout, "");
            }
            let idx = idx % col_info.widths.len();
            device.update_info();
            let printed_str = 
                if self.contains_whitespaced_names {
                    device.quoted_name_colored("'", " ")
                } else {
                    device.name_colored()
                };
            let padding = " ".repeat((col_info.widths[idx] + extra_char_num - device.name_len() - 2).into());
            let _ = write!(stdout, "{printed_str}{padding}");
        }
        let _ = writeln!(stdout);
    }
}

impl IntoIterator for DeviceList {
    type Item = Device;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}

/// Attempts to set the bluetooth pairable state to the value of 
/// new_state and returns whether the action was successful
pub fn pairable(new_state: bool) -> bool {
    Command::new("bluetoothctl")
        .args(["pairable", if new_state {"on"} else {"off"}])
        .output().is_ok_and(|output| {
            if let Ok(out) = String::from_utf8(output.stdout) {
                return out.contains("succeeded");
            };
            false
        })
}
