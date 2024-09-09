// vim: cc=81
use crate::term_utils;
use std::{
    collections::HashMap,
    io::{self, stdout, IsTerminal, Write}, 
    process::{Command, Output, Stdio}, 
    sync::{Arc, Mutex}, thread,
};
use regex::Regex;

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BLACK: &str = "\x1b[30m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_WHITE: &str = "\x1b[37m";
const ANSI_DEFAULT: &str = "\x1b[39m";
const ANSI_BLACK_BG: &str = "\x1b[40m";
const ANSI_RED_BG: &str = "\x1b[41m";
const ANSI_GREEN_BG: &str = "\x1b[42m";
const ANSI_YELLOW_BG: &str = "\x1b[43m";
const ANSI_BLUE_BG: &str = "\x1b[44m";
const ANSI_MAGENTA_BG: &str = "\x1b[45m";
const ANSI_CYAN_BG: &str = "\x1b[46m";
const ANSI_WHITE_BG: &str = "\x1b[47m";
const ANSI_DEFAULT_BG: &str = "\x1b[49m";

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

    // Allow ANSI code color in output from this struct 
    name_in_color: bool,
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
            name_in_color: true,
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
        let out = String::from_utf8(cmd.stdout).unwrap_or("".to_string());
        for line in out.lines() {
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

    /// Attempts to pair with device
    pub fn pair(&mut self) -> bool {
        println!("Attempting to pair with {}...", self.get_name());
        pairable(true);
        let ret = cli_cmd(vec!["pair", &self.address], |out, err| 
            out.contains("Pairing successful") 
                || err.contains("org.bluez.Error.AlreadyExists")
        );
        pairable(false);
        if ret {
            self.paired = Some(true);
            println!("{} paired.", self.get_name());
        } else {
            println!("Could not pair {}.", self.get_name());
        }
        ret
    }

    /// Unpairs the device
    pub fn unpair(&mut self) -> bool {
        println!("Attempting to remove {}...", self.get_name());
        let success = cli_cmd(vec!["remove", &self.address], |out, _| 
            out.contains("Device has been removed"));
        if success {
            self.paired = Some(false);
            self.connected = Some(false);
            println!("{} unpaired.", self.get_name());
        } else {
            println!("Could not unpair {}.", self.get_name());
        }
        success
    }

    /// Attempts to connect to device
    pub fn connect(&mut self) -> bool {
        println!("Attempting to connect with {}...", self.get_name());
        let success = cli_cmd(vec!["connect", &self.address], |out, _|
            out.contains("Connection successful")
        );
        if success {
            self.connected = Some(true);
            println!("{} connected.", self.get_name());
        } else {
            println!("Could not connect {}.", self.get_name());
        }
        success
    }

    /// Attempts to disconnect from device
    pub fn disconnect(&mut self) -> bool {
        println!("Attempting to disconnect from {}...", self.get_name());
        let success = cli_cmd(vec!["disconnect", &self.address], |out, _|
            out.contains(&("[CHG] Device ".to_owned() + &self.address + " Connected: no"))
        );
        if success {
            self.connected = Some(false);
            println!("{} disconnected.", self.get_name());
        } else {
            println!("Could not disconnect {}.", self.get_name());
        }
        success
    }

    /// ANSI color escape sequence based on device state. 
    pub fn ansi_color_codes(&self) -> &str {
        if !self.name_in_color {
            "" 
        } else if self.paired != Some(true) {
            "\x1b[2;37m" // dim, white
        } else if self.connected == Some(true) {
            "\x1b[1;34m" // Bold, blue
        } else {
            "\x1b[22;39m" // Normal, default
        }
    }

    /// ANSI reset escape sequence if name_in_color is true, "" else.
    pub fn ansi_color_reset(&self) -> &str {
        if self.name_in_color { ANSI_RESET } else { "" }
    }

    /// Returns name. Includes ANSI color codes if name_in_color is true.
    pub fn get_name(&self) -> String {
        format!("{}{}{}", 
            self.ansi_color_codes(), 
            self.name,
            self.ansi_color_reset())
    }

    /// Quoted name if it contains whitespace, otherwise placeholder is added 
    /// instead. Includes ANSI color codes if name_in_color is true.
    pub fn quoted_name(&self, quotes: &str, placeholder: &str) -> String {
        format!("{}{2}{}{}{}",  
            self.ansi_color_codes(), 
            self.name,
            if self.name.contains(char::is_whitespace) { 
                quotes 
            } else { 
                placeholder 
            },
            self.ansi_color_reset())
    }

    /// Will print detailed information about the device.
    pub fn print_info(&self) {
        let mut print_str = format!("{} {}", 
            self.address, self.get_name());
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
        let (ansi_red, ansi_yellow, ansi_green) = if self.name_in_color { 
            (ANSI_RED, ANSI_YELLOW, ANSI_GREEN)
        } else { 
            ("", "", "") 
        };
        let ansi_reset = self.ansi_color_reset();
        for (prefix, property) in print_props {
            print_str = print_str + &match property {
                InfoType::OptString(Some(propval)) => 
                    format!("{prefix}{propval}"),
                InfoType::OptBoolean(Some(propval)) => 
                    format!("{prefix}{}{}{ansi_reset}", 
                        if *propval { ansi_green } else { ansi_red },
                        if *propval { "yes" } else { "no" }),
                InfoType::OptBattery(Some(percentage)) => 
                    format!("{prefix}{}{}{ansi_reset}", match percentage {
                        70.. => ansi_green, 
                        30.. => ansi_yellow,
                        _ => ansi_red, 
                    }, percentage),
                _ => String::new(),
            }
        }
        println!("{print_str}");
    }

    /// Returns the length of the device name (as an u8 because
    /// the bluetooth specification limits name length to 248.
    /// See Section 6.23: https://www.bluetooth.com/specifications/core54-html/)
    pub fn name_len(&self) -> u8 {
        self.name.chars().count().try_into()
            .expect("Name length should adhere to bluetooth specification")
    }

    /// Sets whether strings returned by name functions will be colored with 
    /// ANSI color codes
    pub fn set_name_in_color(&mut self, val: bool) {
        self.name_in_color = val;
    }
}

/// Macro for DeviceList, used to asyncronously call a method on all devices in the list
/// and return the sum of the return values of the successful method calls (usuallly
/// evaluating to the amount of devices paired or similar)
macro_rules! _async_all_devices {
    ($func:ident, $x:ident) => { 
        pub fn $func(&self) -> i32 {
            let mut threads = Vec::new();
            for device in &self.devices {
                let device = Arc::clone(&device);
                threads.push(thread::spawn(move || {
                    let mut device = device.lock().expect("Mutex should not be poisoned.");
                    i32::from(device.$x())
                }));
            }
            let mut ret_count: i32 = 0;
            for join_handle in threads {
                if let Ok(thread_ret) = join_handle.join() {
                    ret_count += thread_ret;
                }
            }
            ret_count
        }
    };
}

type Devices = Vec<Arc<Mutex<Device>>>;

#[derive(Debug)]
pub struct DeviceList {
    devices: Devices,

    // Following properties are saved for output
    quote_names: bool,
    print_in_color: bool,
    max_name_len: u8,
    min_name_len: u8,
}

pub enum FilterBehaviour {
    Full,
    Contains,
    FullRegex,
    ContainsRegex,
}

impl DeviceList {
    /// Create a new empty device list
    pub fn new() -> DeviceList {
        DeviceList { 
            devices: Vec::new(), 
            quote_names: false, 
            print_in_color: true,
            max_name_len: 0, 
            min_name_len: 0 
        }
    }

    /// Fills the device list with devices, optionally scanning for unpaired
    /// devices for scan_secs seconds.
    pub fn fill(&mut self, scan_secs : Option<u32>) -> &mut DeviceList {
        let mut devices_args = vec!["devices"];

        if let Some(scan_secs) = scan_secs {
            let do_print = io::stdout().is_terminal() && self.print_in_color;
            if do_print {
                print!("\x1b[2;37mScanning for devices...{ANSI_RESET}");
                let _ = stdout().flush();
            }
            // bluetoothctl scan for unpaired nearby devices
            let _ = Command::new("bluetoothctl")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .args(["--timeout", &scan_secs.to_string(), "scan", "on"])
                .status();
            if do_print {
                print!("\x1b[1K\r");
            }
        } else {
            devices_args.push("Paired");
        }
        let bluetoothctl_output : Output = Command::new("bluetoothctl")
            .args(devices_args)
            .output()
            .expect("failed to execute bluetoothctl devices");

        let output_str = String::from_utf8(bluetoothctl_output.stdout)
            .unwrap_or(String::new());
        for line in output_str.lines() {
            let mut split = line.splitn(3, ' ');
            // First should always be "Device" and line is therefore invalid if 
            // not (for example by delayed device change notifications from scan)
            if split.next() != Some("Device") {
                continue;
            }
            if let (Some(address), Some(name)) = (split.next(), split.next()) {
                self.quote_names |= name.contains(char::is_whitespace);
                let mut device = Device::new(address, name);
                device.name_in_color = self.print_in_color;
                self.max_name_len = self.max_name_len.max(device.name_len());
                self.min_name_len = self.max_name_len.min(device.name_len());
                self.devices.push(Arc::new(Mutex::new(device)));
            }
        }
        self.quote_names &= stdout().lock().is_terminal();
        self
    }

    /// Returns a filtered device list
    pub fn filtered<F>(&self, filter: F) -> DeviceList 
        where F: Fn(&Device) -> bool {
        let mut retval = DeviceList::new();
        for device_ref in &self.devices {
            let mut matches = false;
            if let Ok(device) = device_ref.lock() {
                matches = filter(&device);
            }
            if matches {
                retval.devices.push(Arc::clone(&device_ref));
            }
        }
        retval
    } 

    /// Returns devices in device list with name matching the filterstr, with 
    /// "matching" defined according to behaviour.
    pub fn filtered_name(&self, filterstr : &str, behaviour: FilterBehaviour) -> DeviceList {
        match behaviour {
            FilterBehaviour::Full => self.filtered_name_full(filterstr),
            FilterBehaviour::Contains => self.filtered_name_contains(filterstr),
            FilterBehaviour::FullRegex => self.filtered_name_full_regex(filterstr),
            FilterBehaviour::ContainsRegex => self.filtered_name_contains_regex(filterstr),
        }
    }
    /// Returns devices in device list with given name
    pub fn filtered_name_full(&self, name : &str) -> DeviceList {
        self.filtered(|device| device.name == name)
    }

    /// Returns devices in device list with name containing substr
    pub fn filtered_name_contains(&self, substr : &str) -> DeviceList {
        self.filtered(|device| device.name.contains(substr))
    }

    /// Returns devices in device list with name matching regex.
    pub fn filtered_name_full_regex(&self, regex: &str) -> DeviceList {
        match Regex::new(regex) {
            Ok(re) => self.filtered(|device| re.is_match(&device.name)),
            Err(_) => DeviceList::new(),
        }
    }

    /// Returns devices in device list with name containing a match for the regex.
    pub fn filtered_name_contains_regex(&self, regex: &str) -> DeviceList {
        match Regex::new(regex) {
            Ok(re) => self.filtered(|device| re.find(&device.name).is_some()),
            Err(_) => DeviceList::new(),
        }
    }

    /// Returns the name of the device with decorations depending on state of self
    pub fn correctly_quoted_device_name(&self, device: &Device) -> String {
        if self.quote_names {
            device.quoted_name("'", " ")
        } else {
            device.get_name()
        }
    }

    pub fn print(&mut self, linewise: bool, long_output: bool) {
        if !linewise && !long_output {
            self.print_lines();
        } else if linewise {
            self.print_fullline();
        } else if long_output {
            self.print_long();
        }
    }

    /// Prints each device on its own line (similar to GNU ls -1)
    pub fn print_fullline(&mut self) {
        let mut stdout = stdout().lock();
        for device in &self.devices {
            let mut device = device.lock().expect("Mutex should not be poisoned.");
            device.update_info();
            let _ = writeln!(stdout, "{}",
                self.correctly_quoted_device_name(&device));
        }
    }

    /// Prints each device in long format (on its own line) similar to GNU ls -l
    pub fn print_long(&mut self) {
        let mut stdout = stdout().lock();
        for device in &self.devices {
            let mut device = device.lock().expect("Mutex should not be poisoned.");
            device.update_info();
            let _ = writeln!(stdout, "{} {}", &device.address, 
                self.correctly_quoted_device_name(&device));
        }
    }

    /// Prints multiple devices per line similar to GNU ls -x
    pub fn print_lines(&mut self) {
        // First find highest amount of possible columns and the best fit column 
        // widths
        #[derive(Debug)]
        struct ColsInfo { widths: Vec<u8>, total_w: u16 }
        let max_w: u16 = match term_utils::get_termsize() {
            Some(size) => size.cols,
            _ => 80,
        }.try_into().unwrap_or(80);
        // Checked div prevents divide by zero for empty names
        // Lower bound: Assume all names as long as longest
        let min_cols = max_w.checked_div(self.max_name_len.into()).unwrap_or(0);
        // Upper bound: Assume all names as long as shortest
        let max_cols = max_w.checked_div(self.min_name_len.into()).unwrap_or(max_w);
        // Fallback to print_line if max_name_len > max_w
        // or max_name_len == 0 for the sake of simplicity
        if min_cols == 0 {
            self.print_fullline();
            return;
        }

        // If there are whitespaced names, also account for space used by quotes
        let extra_char_num = 2 + 2 * u8::from(self.quote_names);
        // Infos for every column amount considered
        let mut col_infos: Vec<ColsInfo> = Vec::new();
        col_infos.reserve((max_cols + 1 - min_cols).try_into().unwrap_or(0));
        for cols_num in min_cols..=max_cols {
            col_infos.push(ColsInfo { 
                widths: vec![0; cols_num.into()], 
                total_w: (u16::from(extra_char_num) * cols_num)
                    .try_into().unwrap_or(0),
            })
        };

        for (idx, device) in self.devices.iter().enumerate() {
            let device = device.lock().expect("Mutex should not be poisoned.");
            let device_name_len = device.name_len();
            for (add_cols, col_info) in col_infos.iter_mut().enumerate() {
                // This amount of device columns has already been proven 
                // unusable. Skip to next column amount option
                if col_info.total_w > max_w {
                    break;
                }

                // Calculate column device would be displayed in 
                // add_cols + min_cols is amount of columns
                let idx = idx % (add_cols + usize::from(min_cols));
                if col_info.widths[idx] < device_name_len {
                    let size_incr = device_name_len - col_info.widths[idx];
                    col_info.widths[idx] += size_incr;
                    col_info.total_w += u16::from(size_incr);
                }
            }
        }
        
        // Find highest amount of columns with valid display width
        let mut col_info = &ColsInfo { 
            widths: vec![self.max_name_len], 
            total_w: max_w 
        };
        for candidate in col_infos.iter().rev() {
            if candidate.total_w <= max_w {
                col_info = candidate;
                break;
            };
        };
        // Finally, print
        let mut stdout = stdout().lock();
        for (idx, device) in self.devices.iter().enumerate() {
            let mut device = device.lock().expect("Mutex should not be poisoned.");
            // Output newline when idx 0 is reached (except for first line, 
            // where newline is assumed to already be present)
            if idx != 0 && idx % col_info.widths.len() == 0 {
                let _ = writeln!(stdout, "");
            }
            let idx = idx % col_info.widths.len();
            device.update_info();
            let printed_str = self.correctly_quoted_device_name(&device);
            let padding = " ".repeat(
                (col_info.widths[idx] - device.name_len())
                .into());
            let _ = write!(stdout, "{printed_str}{padding}  ");
        }
        let _ = writeln!(stdout);
    }

    /// Calls print_info on all devices
    pub fn print_info_all(&self) {
        for device in &self.devices {
            let mut device = device.lock().expect("Mutex should not be poisoned.");
            device.update_info();
            device.print_info();
        }
    }
    
    _async_all_devices!(pair_all, pair);
    _async_all_devices!(unpair_all, unpair);
    _async_all_devices!(connect_all, connect);
    _async_all_devices!(disconnect_all, disconnect);

    /// Sets whether quotes will be added if there is a
    /// device name containing whitespace
    pub fn set_quote_names(&mut self, val: bool) {
        self.quote_names = val;
    }

    /// Sets whether output will be colored with ANSI color codes
    pub fn set_print_in_color(&mut self, val: bool) {
        self.print_in_color = val;
    }
}

impl IntoIterator for DeviceList {
    type Item = Arc<Mutex<Device>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}

/// Executes a bluetoothctl command, and calls output_fn(stdout, stderr) for 
/// the returned success value
fn cli_cmd<F>(args: Vec<&str>, output_fn: F) -> bool
    where F: Fn(String, String) -> bool
    {
    Command::new("bluetoothctl")
        .args(args)
        .output().is_ok_and(|output| {
            let out = String::from_utf8(output.stdout).unwrap_or("".to_string());
            let err = String::from_utf8(output.stderr).unwrap_or("".to_string());
            output_fn(out, err)
        })
}


/// Attempts to set the bluetooth pairable state to the value of 
/// new_state and returns whether the action was successful
pub fn pairable(new_state: bool) -> bool {
    cli_cmd(vec!["pairable", if new_state {"on"} else {"off"}], |out, _| 
        out.contains("succeeded"))
}
