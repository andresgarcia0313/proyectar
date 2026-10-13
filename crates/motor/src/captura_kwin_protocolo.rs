//! Client bindings for the two KDE protocols vendored in `protocolos/`.
#![allow(dead_code, non_upper_case_globals, non_camel_case_types, unused_imports, clippy::all)]

pub mod screencast {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocolos/zkde-screencast-unstable-v1.xml");
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocolos/zkde-screencast-unstable-v1.xml");
}

pub mod orden {
    use wayland_client;

    pub mod __interfaces {
        wayland_scanner::generate_interfaces!("protocolos/kde-output-order-v1.xml");
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocolos/kde-output-order-v1.xml");
}
