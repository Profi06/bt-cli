// vim: cc=81
pub mod agent_manager;
pub mod device;
pub mod adapter;

use super::devices::Device;
use super::{BluetoothManager, Devices};
use std::{
    collections::HashMap,
    sync::{ Arc, Mutex },
    time::Duration,
};
use dbus::{
    Path,
    arg::RefArg,
    blocking::{
        Connection, Proxy,
        stdintf::org_freedesktop_dbus::ObjectManager,
    }
};
use device::OrgBluezDevice1;
use adapter::OrgBluezAdapter1;

pub const BLUEZ_DBUS: &str = "org.bluez";
pub const ADAPTER_INTERFACE: &str = "org.bluez.Adapter1";
pub const DEVICE_INTERFACE: &str = "org.bluez.Device1";
pub const BATTERY_INTERFACE: &str = "org.bluez.Battery1";
const DBUS_TIMEOUT: Duration = Duration::new(5, 0);

macro_rules! propmap_get {
    ($propmap: ident, $key: expr, $ty: ty) => {
        $propmap.get($key).and_then(|refarg| {
            refarg.as_any().downcast_ref::<$ty>()
        }).and_then(|reference| Some(reference.to_owned()))
    };
}

pub struct DBusBluetoothManager {
    connection: Connection,
    address_dbus_paths: HashMap<String, Path<'static>>,
    devices: Devices<Self>,
    adapter_paths: Vec<Path<'static>>,
}

impl DBusBluetoothManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let connection = Connection::new_system()?;
        Ok(Self { 
            connection, 
            address_dbus_paths: HashMap::new(),
            devices: Vec::new(), 
            adapter_paths: Vec::new(),
        })
    }
    fn _create_device_proxy<'a: 'b, 'b>(&'a self, address: &'b str) 
        -> Option<Proxy<'b, &'a Connection>> 
    {
        self.address_dbus_paths.get(address).and_then(|path|
            Some(self.connection.with_proxy(BLUEZ_DBUS, path, 
                DBUS_TIMEOUT))
        )
    }
}

impl BluetoothManager for DBusBluetoothManager {
    fn update(&mut self) {
        self.devices = self.connection
            .with_proxy(BLUEZ_DBUS, "/", DBUS_TIMEOUT)
            .get_managed_objects()
            .map_or(Vec::new(), |objects| {
                let mut devices = Vec::new();
                for (path, interfaces) in objects {
                    if let Some(d_props) = interfaces.get(DEVICE_INTERFACE) {
                        let address = propmap_get!(d_props, "Address", String)
                            .expect("Address is required");
                        // alias is used for device.name, not device.name
                        let alias = propmap_get!(d_props, "Alias", String)
                            .expect("Alias is required");
                        let paired = propmap_get!(d_props, "Paired", bool)
                            .expect("Paired is required");
                        let bonded = propmap_get!(d_props, "Bonded", bool)
                            .expect("Bonded is required");
                        let trusted = propmap_get!(d_props, "Trusted", bool)
                            .expect("Trusted is required");
                        let blocked = propmap_get!(d_props, "Blocked", bool)
                            .expect("Blocked is required");
                        let connected = propmap_get!(d_props, "Connected", bool)
                            .expect("Connected is required");
                        let name = propmap_get!(d_props, "Name", String);
                        let icon = propmap_get!(d_props, "Icon", String);

                        let battery = interfaces.get(BATTERY_INTERFACE)
                            .and_then(|battery_props| {
                                propmap_get!(battery_props, "Battery", u8)
                            });
                        self.address_dbus_paths.insert(address.clone(), path);
                        let mut device = Device::new( 
                            address,
                            alias,

                            paired,
                            bonded,
                            trusted,
                            blocked,
                            connected,
                        );
                        device.remote_name = name;
                        device.icon = icon;
                        device.battery = battery;
                        let wrapped_device = Arc::new(Mutex::new(device));
                        devices.push(Arc::clone(&wrapped_device));
                    };
                }
                devices
            });
    }

    fn get_all_devices(&self) -> Devices<Self> {
        Vec::from_iter(self.devices.iter().map(|wrapped_device| {
            Arc::clone(wrapped_device)
        }))
    }

    fn set_pairable(&self, pairable: bool) {
        pairable;
        todo!()
    }

    fn scan(&self, duration: &Duration) {
        duration;
        todo!()
    }


    fn pair_device(&self, address: &str) -> bool {
        self._create_device_proxy(address).is_some_and(|proxy| 
            match proxy.pair() {
                Ok(_) => true,
                // Also return true if the device is already paired
                Err(error) => 
                    error.name() == Some("org.bluez.Error.AlreadyExists")
            })
    }

    fn unpair_device(&self, address: &str) {
        // Get DBus Path to device
        if let Some(d_path) = self.address_dbus_paths.get(address)
            .and_then(|path| Path::new(path.to_string()).ok()) {
            // Get adapter that manages device via proxy
            let d_proxy = self.connection
                .with_proxy(BLUEZ_DBUS, &d_path, DBUS_TIMEOUT);
            if let Ok(path) = d_proxy.adapter() {
                // Disconnect device from its adapter
                self.connection
                    .with_proxy(BLUEZ_DBUS, path, DBUS_TIMEOUT)
                    .remove_device(d_path);
            };
        };
    }

    fn connect_device(&self, address: &str) -> bool {
        self._create_device_proxy(address).is_some_and(|proxy| 
            match proxy.connect() {
                Ok(_) => true,
                // Also return true if the device is already connected
                Err(error) => 
                    error.name() == Some("org.bluez.Error.AlreadyConnected")
            })
    }


    fn disconnect_device(&self, address: &str) {
        if let Some(proxy) = self._create_device_proxy(address) {
            proxy.disconnect();
        };
    }
}

