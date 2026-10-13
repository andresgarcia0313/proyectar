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
