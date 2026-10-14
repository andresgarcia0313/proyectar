use super::{crc32, instalar, CABECERA};
use crate::medios_audio::audio;
use crate::medios_cadena::{PID_AUDIO, PID_PMT};
use crate::medios_lpcm_ts::DESCRIPTOR;
use gstreamer as gst;
use gstreamer::prelude::*;
use wfd::AudioCodec;

fn pid(p: &[u8]) -> u32 {
    (u32::from(p[1] & 0x1F) << 8) | u32::from(p[2])
}

fn carga(p: &[u8]) -> &[u8] {
    &p[4 + if p[3] & 0x20 != 0 { 1 + usize::from(p[4]) } else { 0 }..]
}

/// The real audio branch (30 blocks of tone, cut in odd sizes, in place of the silence) through mpegtsmux to a file.
fn transporte() -> Vec<u8> {
    gst::init().unwrap();
    let archivo = std::env::temp_dir().join(format!("proyectar-lpcm-{}.ts", std::process::id()));
    let rama = audio(Some(AudioCodec::Lpcm), false)
        .replace("wave=silence is-live=true", "wave=sine num-buffers=45 samplesperbuffer=320");
    let texto = format!("{rama} mpegtsmux name=mux alignment=7 ! filesink location={}", archivo.display());
    let tuberia = gst::parse::launch(&texto).unwrap().downcast::<gst::Pipeline>().unwrap();
    let programa = gst::Structure::builder("program_map")
        .field(format!("sink_{PID_AUDIO}"), 1i32)
        .field("PMT_1", PID_PMT)
        .build();
    tuberia.by_name("mux").unwrap().set_property("prog-map", &programa);
    instalar(&tuberia).unwrap();
    tuberia.set_state(gst::State::Playing).unwrap();
    let fin = [gst::MessageType::Eos, gst::MessageType::Error];
    let msg = tuberia.bus().unwrap().timed_pop_filtered(gst::ClockTime::from_seconds(10), &fin);
    tuberia.set_state(gst::State::Null).unwrap();
    assert!(matches!(msg.map(|m| m.type_()), Some(gst::MessageType::Eos)), "la tubería no terminó bien");
    let datos = std::fs::read(&archivo).unwrap();
    let _ = std::fs::remove_file(&archivo);
    datos
}

#[test]
fn el_lpcm_sale_como_lo_pide_miracast() {
    let ts = transporte();
    let paquetes: Vec<&[u8]> = ts.chunks_exact(188).filter(|p| p[1] & 0x40 != 0).collect();

    let pmt = carga(paquetes.iter().copied().find(|p| pid(p) == PID_PMT).expect("sin PMT"));
    let s = 1 + usize::from(pmt[0]);
    let largo = (usize::from(pmt[s + 1] & 0xF) << 8) | usize::from(pmt[s + 2]);
    assert_eq!(crc32(&pmt[s..s + 3 + largo]), 0, "CRC de la PMT inválido");
    let es = s + 12 + ((usize::from(pmt[s + 10] & 0xF) << 8) | usize::from(pmt[s + 11]));
    assert_eq!(pmt[es], 0x83, "tipo de flujo");
    assert_eq!((u32::from(pmt[es + 1] & 0x1F) << 8) | u32::from(pmt[es + 2]), PID_AUDIO);
    assert_eq!(pmt[es + 4], 4, "largo del descriptor");
    assert_eq!(pmt[es + 5..es + 9], DESCRIPTOR);

    let pes: Vec<&[u8]> = paquetes.iter().copied().filter(|p| pid(p) == PID_AUDIO).map(carga).collect();
    assert!(pes.len() > 10, "pocos PES de audio: {}", pes.len());
    for c in pes {
        assert_eq!(c[..4], [0, 0, 1, 0xBD], "flujo privado 1");
        assert_eq!(c[7] & 1, 0, "sin extensión");
        let inicio = 9 + usize::from(c[8]);
        assert_eq!(c[inicio..inicio + 4], CABECERA);
        let largo = (usize::from(c[4]) << 8) | usize::from(c[5]);
        assert_eq!(largo, 3 + usize::from(c[8]) + 4 + 1920, "10 ms por PES");
    }
}

#[test]
fn el_crc_de_una_seccion_firmada_da_cero() {
    let mut seccion = vec![0x02, 0xB0, 0x0D, 0, 1, 0xC1, 0, 0, 0xF0, 0, 0xF0, 0];
    let crc = crc32(&seccion);
    seccion.extend_from_slice(&crc.to_be_bytes());
    assert_eq!(crc32(&seccion), 0);
    assert_eq!(crc32(&[]), 0xFFFF_FFFF);
}
