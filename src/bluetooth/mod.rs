// vim: cc=81
pub mod devices;
pub mod bluez;

use std::time::Duration;

pub use devices::{ DeviceList, Devices, Device};

pub trait BluetoothManager {
    /// Updates the BluetoothManager lists of devices and adapters
    /// Note that created devices may not have their bluetooth_manager set this
    /// instance. To set their bluetooth_manager, add them to a DeviceList.
    fn update(&mut self);
    /// Returns all Devices
    fn get_all_devices(&self) -> Devices<Self> where Self: Sized;
    /// Sets whether the host machine is pairable.
    fn set_pairable(&self, pairable: bool);
    /// Scans for pairable devices for a given duration
    fn scan(&self, duration: &Duration);

    /// Attempts to pair a device. The returned value indicates whether the
    /// device is now paired, also returning true it was already paired.
    fn pair_device(&self, address: &str) -> bool;
    /// Unpairs a device.
    fn unpair_device(&self, address: &str);
    /// Attempts to connect a device. The returned value indicates whether the
    /// device is now connected, also returning true it was already connected.
    fn connect_device(&self, address: &str) -> bool;
    /// Disconnects a device.
    fn disconnect_device(&self, address: &str);
}
