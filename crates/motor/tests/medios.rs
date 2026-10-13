//! Media pipeline against a local stand-in receiver. Silent: the audio is a silence generator.
#[path = "medios/ayuda.rs"]
mod ayuda;
#[path = "medios/receptor.rs"]
mod receptor;
use ayuda::{iniciar, medir_latencia, modo};
use gstreamer::prelude::*;
use motor::medios::Codificador;
use motor::Fallo;
use receptor::Receptor;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

async fn probar(cod: Codificador, m: wfd::VideoMode, segundos: u64) {
    let r = Receptor::nuevo();
    let medios = iniciar(&r, m, cod).expect("iniciar");
    let lat = medir_latencia(&medios);
    medios.estadisticas();
    tokio::time::sleep(Duration::from_secs(segundos)).await;
    let est = medios.estadisticas();
    let mut v = lat.lock().unwrap().clone();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |q: f64| {
        v.get(((v.len() as f64 * q) as usize).min(v.len().saturating_sub(1))).copied().unwrap_or(f64::NAN)
    };
    eprintln!(
        "{cod:?} {}x{}@{}: fps={:.1} kbps={} video={} claves={} audio_ts={} pmt={} codificacion ms mediana={:.1} p95={:.1} max={:.1}",
        m.width, m.height, m.fps, est.fps, est.kbps, r.cuentas.video.load(Relaxed),
        r.cuentas.claves.load(Relaxed), r.cuentas.ts_audio.load(Relaxed), r.cuentas.ts_pmt.load(Relaxed), p(0.5), p(0.95), p(1.0)
    );
    assert!(r.cuentas.video.load(Relaxed) > (m.fps as u64 * (segundos - 1)) / 2, "pocos cuadros H.264");
    // AAC is checked in the transport stream: one PES per frame on PID 0x1100, PMT on 0x1000.
    assert!(r.cuentas.ts_audio.load(Relaxed) > 30 * (segundos - 2), "sin AAC en el TS");
    assert!(r.cuentas.ts_pmt.load(Relaxed) >= 1, "sin PMT en el PID 0x1000");
    assert!(r.cuentas.claves.load(Relaxed) >= 1, "sin cuadro clave");
    assert_eq!(est.gpu, cod.es_gpu());
    // Any bus ERROR ends a real session: none may appear while streaming.
    let error = tokio::time::timeout(Duration::from_millis(50), medios.fallo()).await;
    assert!(error.is_err(), "error en el bus: {error:?}");
    assert!(est.fps > m.fps as f32 * 0.6 && est.kbps > 100, "{est:?}");
    drop(medios);
}

#[tokio::test(flavor = "multi_thread")]
async fn gpu_1080p30_llega_h264_y_aac() {
    probar(Codificador::Gpu, modo(1920, 1080, 30, 1), 4).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn gpu_720p60_perfil_high() {
    probar(Codificador::Gpu, modo(1280, 720, 60, 2), 4).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn respaldo_openh264_y_x264() {
    probar(Codificador::OpenH264, modo(1280, 720, 30, 1), 4).await;
    probar(Codificador::X264, modo(1280, 720, 30, 1), 4).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn pedir_idr_fuerza_cuadros_clave() {
    let r = Receptor::nuevo();
    let medios = iniciar(&r, modo(1280, 720, 30, 1), Codificador::Gpu).unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;
    let antes = r.cuentas.claves.load(Relaxed);
    for _ in 0..4 {
        medios.pedir_idr();
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let nuevas = r.cuentas.claves.load(Relaxed) - antes;
    assert!(nuevas >= 3, "el GOP de 1 s solo daria 1; llegaron {nuevas}");
}

#[tokio::test(flavor = "multi_thread")]
async fn error_del_codificador_se_clasifica() {
    let r = Receptor::nuevo();
    let medios = iniciar(&r, modo(1280, 720, 30, 1), Codificador::Gpu).unwrap();
    let cod = medios.tuberia().by_name("cod").unwrap();
    let bus = medios.tuberia().bus().unwrap();
    let error =
        gstreamer::message::Error::builder(gstreamer::CoreError::Failed, "simulado").src(&cod).build();
    bus.post(error).unwrap();
    let fallo = tokio::time::timeout(Duration::from_secs(5), medios.fallo()).await.expect("fallo");
    assert!(matches!(fallo, Fallo::Codificador { codificador: "vah264lpenc", .. }), "{fallo:?}");
    assert_eq!(fallo.recovery(), motor::Accion::OtroCodificador);
}

#[tokio::test(flavor = "multi_thread")]
async fn error_de_la_fuente_es_captura_y_cadena_de_respaldo() {
    let r = Receptor::nuevo();
    let medios = iniciar(&r, modo(1280, 720, 30, 1), Codificador::Gpu).unwrap();
    let fuente = medios.tuberia().by_name("fuente").unwrap();
    let error =
        gstreamer::message::Error::builder(gstreamer::CoreError::Failed, "simulado").src(&fuente).build();
    medios.tuberia().bus().unwrap().post(error).unwrap();
    assert!(matches!(medios.fallo().await, Fallo::Captura(_)));
    assert_eq!(Codificador::Gpu.siguiente(), Some(Codificador::OpenH264));
    assert_eq!(Codificador::X264.siguiente(), None);
}

#[test]
fn modo_imposible_para_la_gpu_falla_como_codificador() {
    let r = Receptor::nuevo();
    // 8192x8192 is beyond the GPU H.264 encoder: the chain cannot be linked.
    let e = iniciar(&r, modo(8192, 8192, 30, 1), Codificador::Gpu).err().expect("debe fallar");
    assert!(matches!(e, Fallo::Codificador { codificador: "vah264lpenc", .. }), "{e:?}");
}
