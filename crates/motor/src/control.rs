//! The RTSP control socket. It listens only on the Wi-Fi Direct link's address and only the
//! display may use it: nobody on the home or office network can reach the session.

use crate::fallo::{Fallo, Resultado};
use crate::registro;
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

/// Once linked, the TV opens the control session within seconds.
const ESPERA_TV: Duration = Duration::from_secs(30);

/// This laptop's address on the link and the link's prefix length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Red {
    pub ip: Ipv4Addr,
    pub prefijo: u8,
}

impl Red {
    /// Whether `otra` sits on this link: only the display may open the control session.
    pub fn contiene(&self, otra: IpAddr) -> bool {
        let IpAddr::V4(otra) = otra else { return false };
        let mascara = u32::MAX.checked_shl(32 - u32::from(self.prefijo.min(32))).unwrap_or(0);
        u32::from(otra) & mascara == u32::from(self.ip) & mascara
    }
}

/// Waits for the display to connect; anyone else that reaches the port is turned away.
pub async fn esperar_tv(red: Red) -> Resultado<TcpStream> {
    let escucha = escuchar(red).await?;
    let aceptar = async {
        loop {
            let (tcp, de) = escucha.accept().await?;
            let admitido = red.contiene(de.ip());
            registro::anotar("tv_conecta", json!({"desde": de.to_string(), "admitido": admitido}));
            if admitido {
                return Ok(tcp);
            }
        }
    };
    tokio::time::timeout(ESPERA_TV, aceptar).await.map_err(|_| Fallo::SinRespuesta)?
}

/// The address can reach the interface a moment after NetworkManager reports the link:
/// try for a few seconds before giving up.
async fn escuchar(red: Red) -> Resultado<TcpListener> {
    for _ in 0..25 {
        match TcpListener::bind((red.ip, wfd::ie::CONTROL_PORT)).await {
            Err(e) if e.kind() == std::io::ErrorKind::AddrNotAvailable => {
                tokio::time::sleep(Duration::from_millis(200)).await
            }
            otro => return Ok(otro?),
        }
    }
    Err(Fallo::WifiDirect(format!("la dirección {} no llegó a la interfaz Wi-Fi Direct", red.ip)))
}

#[cfg(test)]
mod tests {
    use super::Red;

    #[test]
    fn solo_acepta_la_subred_del_enlace() {
        let red = Red { ip: "192.168.49.1".parse().unwrap(), prefijo: 24 };
        assert!(red.contiene("192.168.49.37".parse().unwrap()));
        assert!(!red.contiene("192.168.1.20".parse().unwrap()));
        assert!(!red.contiene("::1".parse().unwrap()));
        assert!(Red { prefijo: 0, ..red }.contiene("8.8.8.8".parse().unwrap()));
    }

    #[tokio::test]
    async fn direccion_ausente_espera_y_explica() {
        let red = Red { ip: "10.254.254.1".parse().unwrap(), prefijo: 24 };
        let inicio = std::time::Instant::now();
        let fallo = super::escuchar(red).await.err().unwrap().to_string();
        assert!(inicio.elapsed() >= std::time::Duration::from_secs(4), "debe esperar a la interfaz");
        assert!(fallo.contains("10.254.254.1"), "{fallo}");
    }
}
