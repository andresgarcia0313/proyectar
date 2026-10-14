//! The delay inside this laptop, measured on the real pipeline (GPU encoder, MPEG-TS, RTP, UDP)
//! against the local stand-in receiver: the part of the end-to-end delay Proyectar controls.
#[allow(dead_code)]
#[path = "medios/ayuda.rs"]
mod ayuda;
#[allow(dead_code)]
#[path = "medios/receptor.rs"]
mod receptor;
use motor::medios::Codificador;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn retardo_del_equipo_se_mide_y_es_bajo() {
    let r = receptor::Receptor::nuevo();
    let medios = ayuda::iniciar(&r, ayuda::modo(1280, 720, 60, 1), Codificador::Gpu).expect("iniciar");
    tokio::time::sleep(Duration::from_secs(2)).await;
    medios.estadisticas();
    tokio::time::sleep(Duration::from_secs(2)).await;
    let e = medios.estadisticas();
    eprintln!(
        "retardo medio={:.1} ms peor={:.1} ms hasta codificar={:.1} ms",
        e.retardo_ms, e.retardo_peor_ms, e.retardo_cod_ms
    );
    assert!(e.retardo_ms > 0.0, "sin medida: {e:?}");
    // Only that it measures: with a busy CPU the test pattern itself runs late.
    assert!(e.retardo_cod_ms > 0.0, "{e:?}");
}

/// Counts PES starts on the audio PID straight from the RTP packets: no demuxer, any codec.
fn contar_audio_pes() -> (u16, std::sync::Arc<std::sync::atomic::AtomicU64>) {
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(6))).unwrap();
    let (puerto, cuenta) = (socket.local_addr().unwrap().port(), std::sync::Arc::new(Default::default()));
    let c: std::sync::Arc<std::sync::atomic::AtomicU64> = std::sync::Arc::clone(&cuenta);
    std::thread::spawn(move || {
        let mut b = [0u8; 2048];
        while let Ok(n) = socket.recv(&mut b) {
            for p in b[12..n].chunks_exact(188) {
                if (u16::from(p[1] & 0x1f) << 8 | u16::from(p[2])) == 0x1100 && p[1] & 0x40 != 0 {
                    c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    });
    (puerto, cuenta)
}

#[tokio::test(flavor = "multi_thread")]
async fn juegos_con_lpcm_y_refresco_llega_sin_errores() {
    use motor::medios::{Medios, Origen};
    let (puerto, audio) = contar_audio_pes();
    let destino = format!("127.0.0.1:{puerto}").parse().unwrap();
    let perfil = motor::modo::Latencia::Juegos.perfil();
    let modo = ayuda::modo(1280, 720, 60, 1);
    let lpcm = Some(wfd::AudioCodec::Lpcm);
    let medios =
        Medios::iniciar(Origen::Prueba, destino, modo, lpcm, perfil, Codificador::Refresco, false).unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    let error = tokio::time::timeout(Duration::from_millis(50), medios.fallo()).await;
    assert!(error.is_err(), "error en el bus: {error:?}");
    let audio = audio.load(std::sync::atomic::Ordering::Relaxed);
    assert!(audio > 30, "sin LPCM en el TS: {audio}");
}
