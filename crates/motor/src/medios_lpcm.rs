//! Miracast LPCM as the WFD spec and Android's sender carry it: stream type 0x83 with its audio
//! descriptor in the PMT, PES on private_stream_1 (0xBD) and a 4-byte header before every 10 ms of
//! big-endian samples. mpegtsmux knows only DVD LPCM (type 0x8B on stream 0xFD, no header), which
//! the TV takes for an unknown stream and plays nothing; these probes turn the one into the other.

use crate::medios_lpcm_ts::corregir;
use gstreamer as gst;
use gstreamer::prelude::*;

/// sub_stream_id, 6 frames of 80 samples, no emphasis, then 16 bit | 48 kHz | stereo.
pub const CABECERA: [u8; 4] = [0xA0, 6, 0, (2 << 3) | 1];

/// The header goes on each 10 ms block (`lpcm`); the stream is relabelled as it leaves the muxer.
pub fn instalar(tuberia: &gst::Pipeline) -> Option<()> {
    tuberia.by_name("lpcm")?.static_pad("src")?.add_probe(gst::PadProbeType::BUFFER, |_, info| {
        if let Some(gst::PadProbeData::Buffer(b)) = &mut info.data {
            b.make_mut().prepend_memory(gst::Memory::from_slice(CABECERA));
        }
        gst::PadProbeReturn::Ok
    });
    let tipos = gst::PadProbeType::BUFFER | gst::PadProbeType::BUFFER_LIST;
    tuberia.by_name("mux")?.static_pad("src")?.add_probe(tipos, |_, info| {
        let arreglar = |b: &mut gst::Buffer| {
            if let Ok(mut m) = b.make_mut().map_writable() {
                corregir(&mut m);
            }
        };
        match &mut info.data {
            Some(gst::PadProbeData::Buffer(b)) => arreglar(b),
            Some(gst::PadProbeData::BufferList(l)) => {
                l.make_mut().foreach_mut(|mut b, _| {
                    arreglar(&mut b);
                    std::ops::ControlFlow::Continue(Some(b))
                });
            }
            _ => {}
        }
        gst::PadProbeReturn::Ok
    });
    Some(())
}

/// MPEG-2 CRC-32 of a PSI section: over the whole section including its CRC it gives 0.
pub fn crc32(datos: &[u8]) -> u32 {
    datos.iter().fold(0xFFFF_FFFF, |mut crc: u32, &b| {
        crc ^= u32::from(b) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 { (crc << 1) ^ 0x04C1_1DB7 } else { crc << 1 };
        }
        crc
    })
}

#[cfg(test)]
#[path = "medios_lpcm_pruebas.rs"]
mod pruebas;
