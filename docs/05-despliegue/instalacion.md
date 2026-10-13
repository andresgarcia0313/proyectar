---
lang: es-CO
título: Proyectar - Instalación y compilación
versión: 1.0.1
---

# Proyectar - Instalación y compilación

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-12 | Andrés García | Versión inicial: requisitos, dependencias apt, cortafuegos, compilación y comprobaciones |
| 1.0.1 | 2026-10-12 | Andrés García | Limpieza para publicar: requisitos genéricos, sin rutas ni interfaces del equipo |

## 1. Requisitos

| Componente | Versión de referencia | Para qué |
| --- | --- | --- |
| Linux con Plasma Wayland | Plasma 6 (probado en 6.6) | Portal de captura con fuente virtual |
| Rust | 1.85 o superior (rustup) | Compilar el espacio de trabajo |
| Wi-Fi con P2P | Debe admitir P2P-GO (probado con Realtek RTL8852BE) | Wi-Fi Direct |
| NetworkManager | 1.16 o superior (probado con 1.54) | Expone `p2p-dev-<interfaz>` |
| GPU con codificador H.264 | VA-API (Intel, AMD), NVENC o V4L2; probado con Intel Iris Xe e `iHD` | Codificación H.264 de bajo consumo |
| GStreamer | 1.22 o superior (probado con 1.28) | Captura, codificación y empaquetado |
| Qt 6 | El de la distribución | Interfaz (crate `app`, QML con tema Drácula) |

## 2. Dependencias (apt)

Recomendación: simular primero con `-s`, leer la lista de paquetes que se **eliminan** y
instalar siempre con `--no-remove`.

```bash
sudo apt-get install -s --no-remove \
  gstreamer1.0-tools libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-libav gstreamer1.0-pipewire gstreamer1.0-vaapi \
  intel-media-va-driver vainfo xdg-desktop-portal-kde \
  build-essential pkg-config
# si la simulación no elimina nada, repetir sin -s
```

Notas: `vah264lpenc` viene en `gstreamer1.0-plugins-bad`; `mpegtsmux` en el mismo paquete. El codificador
de audio AAC es `fdkaacenc` (plugins-bad, según la distribución) o `avenc_aac` (libav) como reserva. Para
la interfaz: `qt6-base-dev`, `qt6-base-private-dev` y `qt6-declarative-dev`. Equivalentes para Fedora y
Arch en el `README.md`.

## 3. Cortafuegos (ufw)

La TV abre una conexión **hacia** el portátil, así que hay que dejar entrar el TCP 7236 y, cuando el
portátil queda como propietario del grupo (GO, *Group Owner*), el DHCP. Solo en las interfaces del grupo,
que NetworkManager nombra `p2p-*`.

```bash
sudo ufw allow in on p2p-+ to any port 7236 proto tcp   # control RTSP de la TV
sudo ufw allow in on p2p-+ to any port 67 proto udp     # servidor DHCP (somos GO)
sudo ufw allow in on p2p-+ to any port 68 proto udp     # respuestas DHCP (somos cliente)
```

El flujo de video sale por UDP y las respuestas se aceptan por seguimiento de conexión. Para comprobar
las reglas: `sudo ufw status numbered`. Para quitarlas: `sudo ufw delete <número>`.

## 4. Compilar

Perfil `fast`: optimizado, sin LTO (optimización en tiempo de enlace) y con 16 unidades de código, para
iterar rápido; el perfil `release` se reserva para publicar.

```bash
cd proyectar
cargo build --workspace --profile fast        # artefactos en target/fast/
cargo test --workspace                        # pruebas sin hardware
```

## 5. Comprobaciones previas

| Comprobación | Comando | Resultado esperado |
| --- | --- | --- |
| Dispositivo P2P | `nmcli device` | Aparece `p2p-dev-<interfaz>` de tipo `wifi-p2p` |
| Codificador | `gst-inspect-1.0 vah264lpenc` | Muestra el elemento |
| Multiplexor | `gst-inspect-1.0 mpegtsmux` | Muestra el elemento |
| VA-API | `vainfo` | Perfil H.264 con `VAEntrypointEncSliceLP` |
| Portal | `gdbus call --session --dest org.freedesktop.portal.Desktop --object-path /org/freedesktop/portal/desktop --method org.freedesktop.DBus.Properties.GetAll org.freedesktop.portal.ScreenCast` | `AvailableSourceTypes` en 7 |

## 6. Datos del usuario

`~/.config/proyectar/recientes.json` guarda las últimas pantallas, las opciones y los tokens del portal.
Si se daña, la app arranca con la lista vacía. Borrarlo reinicia la memoria de la app, no afecta a la TV.

## 7. Problemas frecuentes

| Síntoma | Causa probable | Solución |
| --- | --- | --- |
| La TV no aparece | No está en Screen Mirroring o NM no ve el dispositivo P2P | Revisar la sección 5 |
| La TV acepta y no hay imagen | TCP 7236 bloqueado | Regla ufw de la sección 3 |
| Enlace sin dirección IP | DHCP bloqueado | Reglas 67 y 68 |
| Pantalla negra con audio | Diseño MPEG-TS rechazado | Ver riesgos en `docs/01-analisis/requisitos.md` |
