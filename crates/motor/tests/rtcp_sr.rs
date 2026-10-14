//! Our RTCP sender reports on a real pipeline: one per second to the TV's RTP port plus one,
//! and the echo in its receiver report turns into the round trip. Silent, no windows.
#[allow(dead_code)]
#[path = "medios/ayuda.rs"]
mod ayuda;
#[allow(dead_code)]
#[path = "medios/receptor.rs"]
mod receptor;
use motor::medios::{Codificador, Medios, Origen};
use motor::modo::Latencia;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

/// RTP and RTCP sockets of a stand-in TV on consecutive ports.
fn puertos_tv() -> (UdpSocket, UdpSocket) {
    loop {
        let rtp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let siguiente = rtp.local_addr().unwrap().port().wrapping_add(1);
        if let Ok(rtcp) = UdpSocket::bind(("127.0.0.1", siguiente)) {
            return (rtp, rtcp);
        }
    }
}

/// RR about SSRC 1 echoing `lsr`, held `dlsr` (1/65536 s); a little jitter, no loss.
fn eco(lsr: u32, dlsr: u32) -> Vec<u8> {
    let mut p =
        vec![0x81, 201, 0, 7, 0x12, 0x34, 0x56, 0x78, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0x1b, 0x58, 0, 0, 0, 90];
    p.extend(lsr.to_be_bytes());
    p.extend(dlsr.to_be_bytes());
    p
}

#[tokio::test(flavor = "multi_thread")]
async fn manda_sr_cada_segundo_y_mide_la_ida_y_vuelta() {
    let (rtp, rtcp) = puertos_tv();
    rtcp.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let perfil = Latencia::Trabajo.perfil();
    let m = ayuda::modo(1280, 720, 30, 1);
    let destino = rtp.local_addr().unwrap();
    let medios = Medios::iniciar(Origen::Prueba, destino, m, None, perfil, Codificador::X264, false).unwrap();
    let mut buf = [0u8; 1500];
    let (n, origen) = rtcp.recv_from(&mut buf).expect("sin SR");
    let primero = Instant::now();
    assert_eq!(
        (n, buf[0], buf[1], origen.port()),
        (28, 0x80, 200, medios.puerto_rtcp()),
        "SR sin bloques desde 5001"
    );
    assert_eq!(u32::from_be_bytes(buf[4..8].try_into().unwrap()), 1, "SSRC de rtpmp2tpay");
    assert!(u32::from_be_bytes(buf[20..24].try_into().unwrap()) > 0, "cuenta paquetes enviados");
    let lsr = u32::from_be_bytes(buf[10..14].try_into().unwrap());
    rtcp.recv_from(&mut buf).expect("sin segundo SR");
    let entre = primero.elapsed();
    assert!(entre > Duration::from_millis(800) && entre < Duration::from_millis(1600), "{entre:?}");
    // The TV held our first report 1 s minus the 5 ms of link we want to see come out.
    let retenido = entre.as_secs_f64() - 0.005;
    rtcp.send_to(&eco(lsr, (retenido * 65_536.0) as u32), ("127.0.0.1", medios.puerto_rtcp())).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let rtt = medios.estadisticas().rtt_ms;
    assert!(rtt > 0.0 && rtt < 300.0, "ida y vuelta {rtt} ms");
}
