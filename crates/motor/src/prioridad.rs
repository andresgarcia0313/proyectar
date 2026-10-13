//! Multimedia priority, the Linux side of Windows' MMCSS: GStreamer's streaming threads ask
//! RealtimeKit for a higher priority so a busy computer does not make the picture stutter.
//! It is a nice level, not realtime: a stuck realtime thread could freeze the desktop.

use tokio::runtime::Handle;
use tokio::sync::OnceCell;

/// RealtimeKit allows down to -15; leave room for PipeWire and the compositor.
const NICE: i32 = -10;

#[zbus::proxy(
    interface = "org.freedesktop.RealtimeKit1",
    default_service = "org.freedesktop.RealtimeKit1",
    default_path = "/org/freedesktop/RealtimeKit1"
)]
trait RealtimeKit {
    fn make_thread_high_priority(&self, thread: u64, priority: i32) -> zbus::Result<()>;
}

static BUS: OnceCell<Option<zbus::Connection>> = OnceCell::const_new();

/// Raises the calling thread. Without RealtimeKit (or a runtime) the thread keeps its
/// normal priority and everything still works.
pub fn subir_este_hilo(runtime: &Option<Handle>) {
    let (Some(runtime), Some(hilo)) = (runtime, hilo_actual()) else { return };
    runtime.spawn(async move {
        let bus = BUS.get_or_init(|| async { zbus::Connection::system().await.ok() }).await;
        if let Some(bus) = bus {
            if let Ok(rtkit) = RealtimeKitProxy::new(bus).await {
                let _ = rtkit.make_thread_high_priority(hilo, NICE).await;
            }
        }
    });
}

/// The kernel id of the calling thread, read from `/proc/thread-self` (`<pid>/task/<tid>`).
fn hilo_actual() -> Option<u64> {
    let ruta = std::fs::read_link("/proc/thread-self").ok()?;
    ruta.file_name()?.to_str()?.parse().ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn conoce_su_propio_hilo() {
        let principal = super::hilo_actual().unwrap();
        let otro = std::thread::spawn(super::hilo_actual).join().unwrap().unwrap();
        assert_ne!(principal, otro);
    }
}
