//! Wi-Fi Display information elements carried in Wi-Fi Direct discovery frames.
//! NetworkManager takes and reports only the subelements (no vendor header):
//! `id(1) length(2) payload`, where subelement 0 is the device information.

/// The RTSP control port every sink connects to (WFD spec, section 6.1.2).
pub const CONTROL_PORT: u16 = 7236;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Source,
    PrimarySink,
    SecondarySink,
    DualRole,
}

impl DeviceType {
    pub fn is_sink(self) -> bool {
        !matches!(self, DeviceType::Source)
    }
}

/// Our subelements: device information (0x0010 = source, available for a session; control
/// port 7236; 200 Mbit/s) and the device name (subelement 10), which TVs show in their list.
pub fn source_ies(name: &str) -> Vec<u8> {
    let [p0, p1] = CONTROL_PORT.to_be_bytes();
    let mut ies = vec![0x00, 0x00, 0x06, 0x00, 0x10, p0, p1, 0x00, 0xc8];
    let name = &name.as_bytes()[..name.len().min(32)];
    ies.extend([0x0a, 0x00, name.len() as u8]);
    ies.extend_from_slice(name);
    ies
}

/// Device type from a peer's subelements; `None` when it does not speak Wi-Fi Display.
pub fn device_type(ies: &[u8]) -> Option<DeviceType> {
    let mut rest = ies;
    while rest.len() >= 3 {
        let (id, len) = (rest[0], usize::from(u16::from_be_bytes([rest[1], rest[2]])));
        let payload = rest.get(3..3 + len)?;
        if id == 0 && payload.len() >= 2 {
            return Some(match payload[1] & 0x03 {
                0 => DeviceType::Source,
                1 => DeviceType::PrimarySink,
                2 => DeviceType::SecondarySink,
                _ => DeviceType::DualRole,
            });
        }
        rest = &rest[3 + len..];
    }
    None
}
