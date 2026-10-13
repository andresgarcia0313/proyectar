//! The slice of NetworkManager's D-Bus API that Wi-Fi Direct needs.

use std::collections::HashMap;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

pub type Settings<'a> = HashMap<&'a str, HashMap<&'a str, Value<'a>>>;

/// `NM_DEVICE_TYPE_WIFI_P2P`.
pub const DEVICE_TYPE_P2P: u32 = 30;
/// `NM_ACTIVE_CONNECTION_STATE_ACTIVATED` and `_DEACTIVATED`.
pub const ACTIVATED: u32 = 2;
pub const DEACTIVATED: u32 = 4;

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn add_and_activate_connection2(
        &self,
        connection: Settings<'_>,
        device: &ObjectPath<'_>,
        specific_object: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<(OwnedObjectPath, OwnedObjectPath, HashMap<String, OwnedValue>)>;

    fn deactivate_connection(&self, active_connection: &ObjectPath<'_>) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait Device {
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Device.WifiP2P",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait WifiP2P {
    fn start_find(&self, options: HashMap<&str, Value<'_>>) -> zbus::Result<()>;
    fn stop_find(&self) -> zbus::Result<()>;
    #[zbus(property)]
    fn peers(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.WifiP2PPeer",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait Peer {
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn hw_address(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
    #[zbus(property, name = "WfdIEs")]
    fn wfd_ies(&self) -> zbus::Result<Vec<u8>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait Active {
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<OwnedObjectPath>;
    #[zbus(signal, name = "StateChanged")]
    fn estado_cambiado(&self, state: u32, reason: u32) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.IP4Config",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait Ip4Config {
    #[zbus(property)]
    fn address_data(&self) -> zbus::Result<Vec<HashMap<String, OwnedValue>>>;
}
