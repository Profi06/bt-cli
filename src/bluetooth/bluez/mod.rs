// vim: cc=81
pub mod adapter;
pub mod agent_manager;
pub mod device;

use super::devices::Device;
use super::{BluetoothManager, Devices};
use crate::utils::ansi::ANSI_RESET;
use adapter::OrgBluezAdapter1;
use dbus::arg::prop_cast;
use dbus::{
    blocking::{stdintf::org_freedesktop_dbus::ObjectManager, Connection, Proxy},
    Path,
};
use device::OrgBluezDevice1;
use std::io::{stdout, Write};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub const BLUEZ_DBUS: &str = "org.bluez";
pub const ADAPTER_INTERFACE: &str = "org.bluez.Adapter1";
pub const DEVICE_INTERFACE: &str = "org.bluez.Device1";
pub const BATTERY_INTERFACE: &str = "org.bluez.Battery1";
const DBUS_TIMEOUT: Duration = Duration::new(60, 0);

pub struct DBusBluetoothManager {
    connection: Connection,
    address_dbus_paths: HashMap<String, Path<'static>>,
    devices: Devices<Self>,
    adapter_paths: Vec<Path<'static>>,
    scan_display_hint: bool,
}

impl DBusBluetoothManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let connection = Connection::new_system()?;
        Ok(Self {
            connection,
            address_dbus_paths: HashMap::new(),
            devices: Vec::new(),
            adapter_paths: Vec::new(),
            scan_display_hint: true,
        })
    }
    fn _create_device_proxy<'a: 'b, 'b>(
        &'a self,
        address: &'b str,
    ) -> Option<Proxy<'b, &'a Connection>> {
        self.address_dbus_paths
            .get(address)
            .and_then(|path| Some(self.connection.with_proxy(BLUEZ_DBUS, path, DBUS_TIMEOUT)))
    }

    pub fn set_scan_display_hint(&mut self, scan_display_hint: bool) {
        self.scan_display_hint = scan_display_hint;
    }
}

impl BluetoothManager for DBusBluetoothManager {
    fn update(&mut self) -> &mut Self {
        self.devices = Vec::new();
        self.adapter_paths = Vec::new();
        if let Ok(objects) = self
            .connection
            .with_proxy(BLUEZ_DBUS, "/", DBUS_TIMEOUT)
            .get_managed_objects()
        {
            for (path, interfaces) in objects {
                if let Some(_) = interfaces.get(ADAPTER_INTERFACE) {
                    self.adapter_paths.push(path);
                } else if let Some(d_props) = interfaces.get(DEVICE_INTERFACE) {
                    let address = prop_cast::<String>(d_props, "Address")
                        .cloned()
                        .expect("Address is required");
                    // alias is used for device.name, not device.name
                    let alias = prop_cast::<String>(d_props, "Alias")
                        .cloned()
                        .expect("Alias is required");
                    let paired = prop_cast::<bool>(d_props, "Paired")
                        .cloned()
                        .expect("Paired is required");
                    let bonded = prop_cast::<bool>(d_props, "Bonded")
                        .cloned()
                        .expect("Bonded is required");
                    let trusted = prop_cast::<bool>(d_props, "Trusted")
                        .cloned()
                        .expect("Trusted is required");
                    let blocked = prop_cast::<bool>(d_props, "Blocked")
                        .cloned()
                        .expect("Blocked is required");
                    let connected = prop_cast::<bool>(d_props, "Connected")
                        .cloned()
                        .expect("Connected is required");
                    let name = prop_cast::<String>(d_props, "Name").cloned();
                    let icon = prop_cast::<String>(d_props, "Icon").cloned();

                    let battery = interfaces.get(BATTERY_INTERFACE).and_then(|battery_props| {
                        prop_cast::<u8>(battery_props, "Battery").cloned()
                    });
                    self.address_dbus_paths.insert(address.clone(), path);
                    let mut device =
                        Device::new(address, alias, paired, bonded, trusted, blocked, connected);
                    device.remote_name = name;
                    device.icon = icon;
                    device.battery = battery;
                    let wrapped_device = Arc::new(Mutex::new(device));
                    self.devices.push(Arc::clone(&wrapped_device));
                };
            }
        }
        self
    }

    fn get_all_devices(&self) -> Devices<Self> {
        Vec::from_iter(
            self.devices
                .iter()
                .map(|wrapped_device| Arc::clone(wrapped_device)),
        )
    }

    fn set_pairable(&self, pairable: bool) {
        pairable;
        todo!()
    }

    fn scan(&self, duration: &Duration) -> &Self {
        for a_path in &self.adapter_paths {
            let proxy = self.connection.with_proxy(BLUEZ_DBUS, a_path, DBUS_TIMEOUT);
            let discovering = proxy.start_discovery().is_ok();
            if discovering {
                if self.scan_display_hint {
                    print!("\x1b[2;37mScanning for devices...{ANSI_RESET}");
                    let _ = stdout().flush();
                }
                thread::sleep(*duration);
                let _ = proxy.stop_discovery();
                if self.scan_display_hint {
                    print!("\x1b[1K\r");
                }
            }
        }
        &self
    }

    fn pair_device(&self, address: &str) -> bool {
        self._create_device_proxy(address)
            .is_some_and(|proxy| match proxy.pair() {
                Ok(_) => true,
                // Also return true if the device is already paired
                Err(error) => error.name() == Some("org.bluez.Error.AlreadyExists"),
            })
    }

    fn unpair_device(&self, address: &str) {
        // Get DBus Path to device
        if let Some(d_path) = self
            .address_dbus_paths
            .get(address)
            .and_then(|path| Path::new(path.to_string()).ok())
        {
            // Get adapter that manages device via proxy
            let d_proxy = self
                .connection
                .with_proxy(BLUEZ_DBUS, &d_path, DBUS_TIMEOUT);
            if let Ok(path) = d_proxy.adapter() {
                // Disconnect device from its adapter
                let _ = self
                    .connection
                    .with_proxy(BLUEZ_DBUS, path, DBUS_TIMEOUT)
                    .remove_device(d_path);
            };
        };
    }

    fn connect_device(&self, address: &str) -> bool {
        self._create_device_proxy(address)
            .is_some_and(|proxy| match proxy.connect() {
                Ok(_) => true,
                // Also return true if the device is already connected
                Err(error) => error.name() == Some("org.bluez.Error.AlreadyConnected"),
            })
    }

    fn disconnect_device(&self, address: &str) {
        if let Some(proxy) = self._create_device_proxy(address) {
            let _ = proxy.disconnect();
        };
    }
}
