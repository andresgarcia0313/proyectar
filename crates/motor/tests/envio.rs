//! The paced sender on a real pipeline: source port, RTCP reports into the statistics, and a
//! dead link classified as a media failure. Silent, no windows; run in series (fixed ports).
#[allow(dead_code)] // shared helpers: this file uses only some of them
#[path = "medios/ayuda.rs"]
mod ayuda;
#[allow(dead_code)]
#[path = "medios/receptor.rs"]
mod receptor;
use ayuda::modo;
use motor::envio::{PUERTO_RTCP, PUERTO_RTP};
use motor::medios::{Codificador, Medios, Origen};
use motor::modo::Latencia;
use motor::Fallo;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

fn iniciar(destino: SocketAddr) -> Medios {
    let perfil = Latencia::Trabajo.perfil();
    Medios::iniciar(Origen::Prueba, destino, modo(1280, 720, 30, 1), None, perfil, Codificador::X264, false)
        .expect("iniciar")
}

/// RR about SSRC 1: 64/256 lost lately, 300 in total, jitter 450 ticks (5 ms); then an SDES.
fn informe() -> Vec<u8> {
    let mut p = vec![0x81, 201, 0, 7, 0x12, 0x34, 0x56, 0x78, 0, 0, 0, 1, 64, 0, 0x01, 0x2c];
    p.extend([0, 0, 0x1b, 0x58, 0, 0, 0x01, 0xc2, 0, 0, 0, 0, 0, 0, 0, 0]);
    p.extend([0x81, 202, 0, 2, 0x12, 0x34, 0x56, 0x78, 1, 2, b't', b'v']);
    p
}

#[tokio::test(flavor = "multi_thread")]
async fn sale_rtp_desde_el_puerto_anunciado() {
    let tv = UdpSocket::bind("127.0.0.1:0").unwrap();
    tv.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let medios = iniciar(tv.local_addr().unwrap());
    let mut buf = [0u8; 2000];
    let (n, origen) = tv.recv_from(&mut buf).expect("sin paquetes RTP");
    assert_eq!(origen.port(), PUERTO_RTP, "el TV manda RTCP al puerto siguiente al anunciado");
    assert_eq!(medios.puerto_rtcp(), PUERTO_RTCP);
    assert_eq!((buf[0], buf[1] & 0x7f, n), (0x80, 33, 12 + 7 * 188), "RTP v2, MP2T, 7 paquetes TS");
}

#[tokio::test(flavor = "multi_thread")]
async fn el_informe_rtcp_llega_a_las_estadisticas() {
    let r = receptor::Receptor::nuevo();
    let medios = iniciar(format!("127.0.0.1:{}", r.puerto).parse().unwrap());
    let tv = UdpSocket::bind("127.0.0.1:0").unwrap();
    tv.send_to(&informe(), ("127.0.0.1", medios.puerto_rtcp())).unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let est = medios.estadisticas();
    assert_eq!((est.perdida_pct, est.perdidos, est.jitter_ms), (25.0, 300, 5.0), "{est:?}");
    let ajeno = UdpSocket::bind("127.0.0.2:0").unwrap();
    let mut limpio = informe();
    limpio[12] = 0;
    ajeno.send_to(&limpio, ("127.0.0.1", medios.puerto_rtcp())).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(medios.estadisticas().perdida_pct, 25.0, "solo cuenta lo que manda el TV");
}

#[tokio::test(flavor = "multi_thread")]
async fn un_envio_imposible_es_fallo_de_medios() {
    // An IPv6 target from the IPv4 socket: every send fails, as with a link that is gone.
    let medios = iniciar("[::1]:9".parse().unwrap());
    let fallo = tokio::time::timeout(Duration::from_secs(5), medios.fallo()).await.expect("sin fallo");
    assert!(matches!(&fallo, Fallo::Medios(d) if d.contains("envío UDP")), "{fallo:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn mide_el_silencio_y_los_cuadros_descartados() {
    let tv = UdpSocket::bind("127.0.0.1:0").unwrap();
    let medios = iniciar(tv.local_addr().unwrap());
    tokio::time::sleep(Duration::from_millis(2600)).await;
    let e = medios.estadisticas();
    assert!(e.audio_medido_s >= 2 && e.audio_senal_s == 0, "el silencio se mide y no cuenta: {e:?}");
    assert!(e.descartes < 15, "la prueba sostiene 30 fps sin botar cuadros: {e:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn al_soltar_medios_los_puertos_quedan_libres() {
    let tv = UdpSocket::bind("127.0.0.1:0").unwrap();
    drop(iniciar(tv.local_addr().unwrap()));
    UdpSocket::bind(("0.0.0.0", PUERTO_RTP)).expect("5000 sigue tomado");
    UdpSocket::bind(("0.0.0.0", PUERTO_RTCP)).expect("5001 sigue tomado");
}
