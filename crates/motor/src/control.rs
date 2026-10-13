//! The RTSP control socket. It listens only on the Wi-Fi Direct link's address and only the
//! display may use it: nobody on the home or office network can reach the session.

use crate::enlace::Red;
use crate::fallo::{Fallo, Resultado};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

/// Once linked, the TV opens the control session within seconds.
const ESPERA_TV: Duration = Duration::from_secs(30);

/// Waits for the display to connect; anyone else that reaches the port is turned away.
pub async fn esperar_tv(red: Red) -> Resultado<TcpStream> {
    let escucha = TcpListener::bind((red.ip, wfd::ie::CONTROL_PORT)).await?;
    let aceptar = async {
        loop {
            let (tcp, de) = escucha.accept().await?;
            if red.contiene(de.ip()) {
                return Ok(tcp);
            }
        }
    };
    tokio::time::timeout(ESPERA_TV, aceptar).await.map_err(|_| Fallo::SinRespuesta)?
}
