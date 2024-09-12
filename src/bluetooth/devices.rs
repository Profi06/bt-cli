// vim: cc=81
use crate::utils::{self, ansi::*};
use regex::Regex;
use std::{
    io::{stdout, Write},
    process::Command,
    sync::Weak,
    sync::{Arc, Mutex},
};

use super::BluetoothManager;

pub struct Device<M: BluetoothManager> {
    pub address: String,
    pub name: String,
    pub bluetooth_manager: Weak<Mutex<M>>,

    pub paired: bool,
    pub bonded: bool,
    pub trusted: bool,
    pub blocked: bool,
    pub connected: bool,

    /// Unlike name this cannot be renamed locally
    pub remote_name: Option<String>,
    pub battery: Option<u8>,
    pub icon: Option<String>,

    // Allow ANSI code color in output from this struct
    name_in_color: bool,
}

enum InfoType<'a> {
    Boolean(&'a bool),
    String(&'a String),
    OptBoolean(&'a Option<bool>),
    OptString(&'a Option<String>),
    OptBattery(&'a Option<u8>),
}

impl<M: BluetoothManager> Device<M> {
    pub fn new(
        address: String,
        name: String,
        paired: bool,
        bonded: bool,
        trusted: bool,
        blocked: bool,
        connected: bool,
    ) -> Device<M> {
        Device {
            address,
            name,
            bluetooth_manager: Weak::<Mutex<M>>::new(),

            paired,
            bonded,
            trusted,
            blocked,
            connected,

            remote_name: None,
            battery: None,
            icon: None,

            name_in_color: true,
        }
    }

    /// Attempts to pair with device
    pub fn pair(&mut self) -> bool {
        println!("Attempting to pair with {}...", self.get_name_colored());
        pairable(true);
        let success = self.bluetooth_manager.upgrade().is_some_and(|bt_man| {
            bt_man
                .lock()
                .expect("Mutex should not be poisoned.")
                .pair_device(&self.address)
        });
        pairable(false);
        if success {
            self.paired = true;
            println!("{} paired.", self.get_name_colored());
        } else {
            println!("Could not pair {}.", self.get_name_colored());
        }
        success
    }

    /// Unpairs the device. Only fails if bluetooth_manager is invalid.
    pub fn unpair(&mut self) -> bool {
        let success = self.bluetooth_manager.upgrade().is_some_and(|bt_man| {
            bt_man
                .lock()
                .expect("Mutex should not be poisoned.")
                .unpair_device(&self.address);
            true
        });
        if success {
            self.paired = false;
            self.connected = false;
            println!("{} unpaired.", self.get_name_colored());
        } else {
            println!("Could not unpair {}.", self.get_name_colored());
        }
        success
    }

    /// Attempts to connect to device
    pub fn connect(&mut self) -> bool {
        println!("Attempting to connect with {}...", self.get_name_colored());
        let success = self.bluetooth_manager.upgrade().is_some_and(|bt_man| {
            bt_man
                .lock()
                .expect("Mutex should not be poisoned.")
                .connect_device(&self.address)
        });
        if success {
            self.connected = true;
            println!("{} connected.", self.get_name_colored());
        } else {
            println!("Could not connect {}.", self.get_name_colored());
        }
        success
    }

    /// Disconnects the device. Only fails if bluetooth_manager is invalid.
    pub fn disconnect(&mut self) -> bool {
        let success = self.bluetooth_manager.upgrade().is_some_and(|bt_man| {
            bt_man
                .lock()
                .expect("Mutex should not be poisoned.")
                .disconnect_device(&self.address);
            true
        });
        if success {
            self.paired = false;
            self.connected = false;
            println!("{} disconnected.", self.get_name_colored());
        } else {
            println!("Could not disconnect {}.", self.get_name_colored());
        }
        success
    }

    /// ANSI color escape sequence based on device state.
    pub fn ansi_color_codes(&self) -> &str {
        if !self.name_in_color {
            ""
        } else if self.paired != true {
            "\x1b[2;37m" // dim, white
        } else if self.connected == true {
            "\x1b[1;34m" // Bold, blue
        } else {
            "\x1b[22;39m" // Normal, default
        }
    }

    /// ANSI reset escape sequence if name_in_color is true, "" else.
    pub fn ansi_color_reset(&self) -> &str {
        if self.name_in_color {
            ANSI_RESET
        } else {
            ""
        }
    }

    /// Returns name. Includes ANSI color codes if name_in_color is true.
    pub fn get_name_colored(&self) -> String {
        format!(
            "{}{}{}",
            self.ansi_color_codes(),
            self.name,
            self.ansi_color_reset()
        )
    }

    /// Quoted name if it contains whitespace, otherwise placeholder is added
    /// instead. Includes ANSI color codes if name_in_color is true.
    pub fn quoted_name(&self, quotes: &str, placeholder: &str) -> String {
        format!(
            "{}{2}{}{}{}",
            self.ansi_color_codes(),
            self.name,
            if self.name.contains(char::is_whitespace) {
                quotes
            } else {
                placeholder
            },
            self.ansi_color_reset()
        )
    }

    /// Will print detailed information about the device.
    pub fn print_info(&self) {
        let mut print_str = format!("{} {}", self.address, self.get_name_colored());
        let print_props = Vec::from([
            ("\n\tPaired: ", InfoType::Boolean(&self.paired)),
            ("\n\tBonded: ", InfoType::Boolean(&self.bonded)),
            ("\n\tTrusted: ", InfoType::Boolean(&self.trusted)),
            ("\n\tBlocked: ", InfoType::Boolean(&self.blocked)),
            ("\n\tConnected: ", InfoType::Boolean(&self.connected)),
            ("\n\tRemote Name: ", InfoType::OptString(&self.remote_name)),
            (
                "\n\tBattery Percentage: ",
                InfoType::OptBattery(&self.battery),
            ),
            ("\n\tIcon: ", InfoType::OptString(&self.icon)),
        ]);
        let (ansi_red, ansi_yellow, ansi_green) = if self.name_in_color {
            (ANSI_RED, ANSI_YELLOW, ANSI_GREEN)
        } else {
            ("", "", "")
        };
        let ansi_reset = self.ansi_color_reset();
        for (prefix, property) in print_props {
            print_str = print_str
                + &match property {
                    InfoType::String(propval) | InfoType::OptString(Some(propval)) => {
                        format!("{prefix}{propval}")
                    }
                    InfoType::Boolean(propval) | InfoType::OptBoolean(Some(propval)) => format!(
                        "{prefix}{}{}{ansi_reset}",
                        if *propval { ansi_green } else { ansi_red },
                        if *propval { "yes" } else { "no" }
                    ),
                    InfoType::OptBattery(Some(percentage)) => format!(
                        "{prefix}{}{}{ansi_reset}",
                        match percentage {
                            70.. => ansi_green,
                            30.. => ansi_yellow,
                            _ => ansi_red,
                        },
                        percentage
                    ),
                    _ => String::new(),
                }
        }
        println!("{print_str}");
    }

    /// Returns the length of the device name (as an u8 because
    /// the bluetooth specification limits name length to 248.
    /// See Section 6.23: https://www.bluetooth.com/specifications/core54-html/)
    pub fn name_len(&self) -> u8 {
        self.name
            .chars()
            .count()
            .try_into()
            .expect("Name length should adhere to bluetooth specification")
    }

    /// Sets whether strings returned by name functions will be colored with
    /// ANSI color codes
    pub fn set_name_in_color(&mut self, val: bool) {
        self.name_in_color = val;
    }

    /// Sets the bluetooth manager of this device to a Weak downgraded from
    /// the passed Arc
    pub fn set_bluetooth_manager(&mut self, bt_man: &Arc<Mutex<M>>) {
        self.bluetooth_manager = Arc::downgrade(bt_man);
    }
}

/// Macro for DeviceList, used to asyncronously call a method on all devices in
/// the list and return the sum of the return values of the successful method
/// calls (usuallly evaluating to the amount of devices paired or similar)
macro_rules! _async_all_devices {
    ($func:ident, $x:ident) => {
        pub fn $func(&self) -> i32 {
            let mut ret_count: i32 = 0;
            for device in &self.devices {
                let mut device = device.lock().expect("Mutex should not be poisoned.");
                ret_count += i32::from(device.$x());
            }
            ret_count
        }
    };
}

pub type Devices<M> = Vec<Arc<Mutex<Device<M>>>>;

pub struct DeviceList<M: BluetoothManager> {
    devices: Devices<M>,
    bluetooth_manager: Arc<Mutex<M>>,

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

impl<M: BluetoothManager> DeviceList<M> {
    /// Create a new empty device list
    pub fn new(bluetooth_manager: Arc<Mutex<M>>) -> DeviceList<M> {
        DeviceList {
            devices: Vec::new(),
            bluetooth_manager,
            quote_names: false,
            print_in_color: true,
            max_name_len: 0,
            min_name_len: 0,
        }
    }

    /// Adds a device to this DeviceList
    pub fn add_device(&mut self, new: Arc<Mutex<Device<M>>>) {
        let mut device = new.lock().expect("Mutex should not be poisoned.");
        device.set_bluetooth_manager(&self.bluetooth_manager);

        self.quote_names |= device.name.contains(char::is_whitespace);
        device.name_in_color = self.print_in_color;
        let name_len = device.name_len();
        self.max_name_len = self.max_name_len.max(name_len);
        self.min_name_len = self.max_name_len.min(name_len);

        drop(device);
        self.devices.push(new);
    }

    /// Fills the device list with devices, optionally scanning for unpaired
    /// devices for scan_secs seconds.
    pub fn fill(&mut self) -> &mut DeviceList<M> {
        let devices = self
            .bluetooth_manager
            .lock()
            .expect("Mutex should not be poisoned.")
            .get_all_devices();
        for wrapped_device in devices {
            self.add_device(wrapped_device)
        }
        self
    }
    /*
        /// Fills the device list with devices, optionally scanning for unpaired
        /// devices for scan_secs seconds.
        pub fn fill(&mut self, scan_secs : Option<u32>) -> &mut DeviceList<M> {
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
                    let mut device = Device::new(address, name);
                    self.devices.push(Arc::new(Mutex::new(device)));
                }
            }
            self.quote_names &= stdout().lock().is_terminal();
            self
        }
    */

    /// Returns a filtered device list
    pub fn filtered<F>(&self, filter: F) -> DeviceList<M>
    where
        F: Fn(&Device<M>) -> bool,
    {
        let mut retval = DeviceList::new(Arc::clone(&self.bluetooth_manager));
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
    pub fn filtered_name(&self, filterstr: &str, behaviour: FilterBehaviour) -> DeviceList<M> {
        match behaviour {
            FilterBehaviour::Full => self.filtered_name_full(filterstr),
            FilterBehaviour::Contains => self.filtered_name_contains(filterstr),
            FilterBehaviour::FullRegex => self.filtered_name_full_regex(filterstr),
            FilterBehaviour::ContainsRegex => self.filtered_name_contains_regex(filterstr),
        }
    }
    /// Returns devices in device list with given name
    pub fn filtered_name_full(&self, name: &str) -> DeviceList<M> {
        self.filtered(|device| device.name == name)
    }

    /// Returns devices in device list with name containing substr
    pub fn filtered_name_contains(&self, substr: &str) -> DeviceList<M> {
        self.filtered(|device| device.name.contains(substr))
    }

    /// Returns devices in device list with name matching regex.
    pub fn filtered_name_full_regex(&self, regex: &str) -> DeviceList<M> {
        match Regex::new(regex) {
            Ok(re) => self.filtered(|device| re.is_match(&device.name)),
            Err(_) => DeviceList::new(Arc::clone(&self.bluetooth_manager)),
        }
    }

    /// Returns devices in device list with name containing a match for the regex.
    pub fn filtered_name_contains_regex(&self, regex: &str) -> DeviceList<M> {
        match Regex::new(regex) {
            Ok(re) => self.filtered(|device| re.find(&device.name).is_some()),
            Err(_) => DeviceList::new(Arc::clone(&self.bluetooth_manager)),
        }
    }

    /// Returns the name of the device with decorations depending on state of self
    pub fn correctly_quoted_device_name(&self, device: &Device<M>) -> String {
        if self.quote_names {
            device.quoted_name("'", " ")
        } else {
            device.get_name_colored()
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
            let device = device.lock().expect("Mutex should not be poisoned.");
            let _ = writeln!(stdout, "{}", self.correctly_quoted_device_name(&device));
        }
    }

    /// Prints each device in long format (on its own line) similar to GNU ls -l
    pub fn print_long(&mut self) {
        let mut stdout = stdout().lock();
        for device in &self.devices {
            let device = device.lock().expect("Mutex should not be poisoned.");
            let _ = writeln!(
                stdout,
                "{} {}",
                &device.address,
                self.correctly_quoted_device_name(&device)
            );
        }
    }

    /// Prints multiple devices per line similar to GNU ls -x
    pub fn print_lines(&mut self) {
        // First find highest amount of possible columns and the best fit column
        // widths
        #[derive(Debug)]
        struct ColsInfo {
            widths: Vec<u8>,
            total_w: u16,
        }
        let max_w: u16 = match utils::get_termsize() {
            Some(size) => size.cols,
            _ => 80,
        }
        .try_into()
        .unwrap_or(80);
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
                    .try_into()
                    .unwrap_or(0),
            })
        }

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
            total_w: max_w,
        };
        for candidate in col_infos.iter().rev() {
            if candidate.total_w <= max_w {
                col_info = candidate;
                break;
            };
        }
        // Finally, print
        let mut stdout = stdout().lock();
        for (idx, device) in self.devices.iter().enumerate() {
            let device = device.lock().expect("Mutex should not be poisoned.");
            // Output newline when idx 0 is reached (except for first line,
            // where newline is assumed to already be present)
            if idx != 0 && idx % col_info.widths.len() == 0 {
                let _ = writeln!(stdout, "");
            }
            let idx = idx % col_info.widths.len();
            let printed_str = self.correctly_quoted_device_name(&device);
            let padding = " ".repeat((col_info.widths[idx] - device.name_len()).into());
            let _ = write!(stdout, "{printed_str}{padding}  ");
        }
        let _ = writeln!(stdout);
    }

    /// Calls print_info on all devices
    pub fn print_info_all(&self) {
        for device in &self.devices {
            let device = device.lock().expect("Mutex should not be poisoned.");
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

impl<M: BluetoothManager> IntoIterator for DeviceList<M> {
    type Item = Arc<Mutex<Device<M>>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}

/// Executes a bluetoothctl command, and calls output_fn(stdout, stderr) for
/// the returned success value
fn cli_cmd<F>(args: Vec<&str>, output_fn: F) -> bool
where
    F: Fn(String, String) -> bool,
{
    Command::new("bluetoothctl")
        .args(args)
        .output()
        .is_ok_and(|output| {
            let out = String::from_utf8(output.stdout).unwrap_or("".to_string());
            let err = String::from_utf8(output.stderr).unwrap_or("".to_string());
            output_fn(out, err)
        })
}

/// Attempts to set the bluetooth pairable state to the value of
/// new_state and returns whether the action was successful
pub fn pairable(new_state: bool) -> bool {
    cli_cmd(
        vec!["pairable", if new_state { "on" } else { "off" }],
        |out, _| out.contains("succeeded"),
    )
}
