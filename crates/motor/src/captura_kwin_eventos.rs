//! KWin's events, folded into [`Estado`].

use super::estado::Estado;
use super::protocolo::orden::kde_output_order_v1::{Event as Orden, KdeOutputOrderV1};
use super::protocolo::screencast::zkde_screencast_stream_unstable_v1::{
    self as flujo, ZkdeScreencastStreamUnstableV1,
};
use super::protocolo::screencast::zkde_screencast_unstable_v1::ZkdeScreencastUnstableV1;
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{delegate_noop, Connection, Dispatch, QueueHandle, WEnum};

type Cola = QueueHandle<Estado>;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Estado {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &Cola,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Estado {
    fn event(
        estado: &mut Self,
        objeto: &wl_output::WlOutput,
        evento: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &Cola,
    ) {
        let Some(salida) = estado.salidas.iter_mut().find(|s| &s.objeto == objeto) else { return };
        match evento {
            wl_output::Event::Name { name } => salida.nombre = Some(name),
            wl_output::Event::Mode { flags: WEnum::Value(f), width, height, .. }
                if f.contains(wl_output::Mode::Current) =>
            {
                salida.modo = Some((width, height));
            }
            _ => {}
        }
    }
}

impl Dispatch<KdeOutputOrderV1, ()> for Estado {
    fn event(estado: &mut Self, _: &KdeOutputOrderV1, evento: Orden, _: &(), _: &Connection, _: &Cola) {
        match evento {
            Orden::Output { output_name } => estado.nuevo_orden.push(output_name),
            Orden::Done => estado.orden = std::mem::take(&mut estado.nuevo_orden),
        }
    }
}

impl Dispatch<ZkdeScreencastStreamUnstableV1, ()> for Estado {
    fn event(
        estado: &mut Self,
        _: &ZkdeScreencastStreamUnstableV1,
        evento: flujo::Event,
        _: &(),
        _: &Connection,
        _: &Cola,
    ) {
        match evento {
            flujo::Event::Created { node } => estado.nodo = Some(node),
            flujo::Event::Failed { error } => estado.fallo = Some(error),
            flujo::Event::Closed => estado.cerrado = true,
            _ => {}
        }
    }
}

delegate_noop!(Estado: ignore ZkdeScreencastUnstableV1);
