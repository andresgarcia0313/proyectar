---
lang: es-CO
título: Proyectar
versión: 0.5.1
---

# Proyectar

[![CI](https://github.com/andresgarcia0313/proyectar/actions/workflows/ci.yml/badge.svg)](https://github.com/andresgarcia0313/proyectar/actions/workflows/ci.yml)

Proyección inalámbrica de pantalla y audio a televisores **Miracast** (Wi-Fi Display) desde Linux, con la
sencillez de Win+K en Windows: abre, elige la TV y listo. Núcleo en Rust, interfaz en Qt 6 con QML y
paleta Drácula.

*English version below: [English](#english).*

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 0.1.0 | 2026-10-12 | Andrés García | Documentación inicial del proyecto |
| 0.1.1 | 2026-10-12 | Andrés García | README bilingüe para publicar: funciones, requisitos, GPU, receptores, instalación, cortafuegos y modo demo |
| 0.2.0 | 2026-10-13 | Andrés García | Nombre del equipo en Wi-Fi Direct (`--sistema`), fases de conexión, diagnóstico y registro de eventos |
| 0.5.0 | 2026-10-13 | Andrés García | Monitor de comportamiento: autodiagnóstico, señales (RTT, audio, descartes, carga), resumen por sesión con veredictos y `proyectar --informe` |
| 0.4.0 | 2026-10-13 | Andrés García | Modo Juegos de baja latencia (LPCM, refresco intra, búfer de 3 cuadros), envío a ritmo constante, informes RTCP del TV, cronómetro para medir |
| 0.3.0 | 2026-10-13 | Andrés García | Salida de audio del TV en KDE, latencia del equipo visible, búfer de decodificación por perfil, monitoreo de radio y CPU, tamaño de ventana recordado |
| 0.5.1 | 2026-10-14 | Andrés García | La búsqueda de pantallas se detiene a los 2 minutos (↻ la reanuda): dejada abierta horas trababa el escaneo Wi-Fi de la tarjeta rtw89 |

## Qué es

Proyectar es el equivalente en Linux del panel "Conectar" de Windows (Win+K). Actúa como **emisor**
Miracast: descubre los televisores cercanos por Wi-Fi Direct, se enlaza con el elegido mediante
NetworkManager, negocia Wi-Fi Display por RTSP y envía la pantalla codificada en H.264 por la GPU, con el
audio, en MPEG-TS sobre RTP. La Wi-Fi normal sigue conectada mientras proyecta.

## Funciones

- **Descubrimiento:** lista las pantallas Miracast cercanas con nombre y señal, sin configurar nada.
- **Tres formas de proyectar:** Duplicar (la pantalla actual), Extender (crea un monitor virtual para la
  TV) y Ventana (solo una ventana).
- **Modos de latencia:** Juegos (720p60, mínima latencia), Trabajo (1080p30) y Videos (1080p60).
- **Audio a la TV:** el sonido del equipo sale por el televisor; si se desactiva, se envía silencio para
  que la TV no corte la sesión.
- **Recientes:** recuerda las últimas pantallas y el permiso de captura, para conectar con un clic.
- **Reconexión automática:** se recupera sola si cae el enlace, la captura o el codificador, y si un
  codificador falla pasa al siguiente de la cadena de respaldo sin cerrar la sesión.

No incluye HDCP (protección de contenido), entrada desde la TV (UIBC) ni Miracast sobre infraestructura
(MICE). Detalle en [requisitos](docs/01-analisis/requisitos.md).

## Requisitos

| Componente | Versión mínima | Notas |
| --- | --- | --- |
| Linux con NetworkManager | 1.16 | Primera versión con dispositivos `wifi-p2p` |
| Tarjeta Wi-Fi con P2P | - | Debe admitir el rol P2P-GO (ver abajo) |
| PipeWire y xdg-desktop-portal | El de la distribución | Captura de pantalla; Extender requiere un portal con fuente virtual (por ejemplo KDE Plasma 6) |
| GStreamer | 1.22 | Con plugins base, good, bad, ugly y libav |
| Qt 6 | 6.4 | Qt Quick y Qt Quick Controls |
| Rust | 1.85 | Solo para compilar (rustup) |

Para saber si la tarjeta Wi-Fi sirve, busque `P2P-GO` en las combinaciones de interfaces:

```bash
iw list | grep -A8 "valid interface combinations"
```

Si la salida menciona `P2P-client` y `P2P-GO`, la tarjeta puede hacer Wi-Fi Direct.

## GPU y codificadores

Proyectar elige el primer codificador H.264 disponible y, si falla, pasa al siguiente:

| Hardware | Elemento de GStreamer | Paquete típico |
| --- | --- | --- |
| Intel y AMD (VA-API) | `vah264lpenc` (bajo consumo), `vah264enc` | plugins-bad + controlador VA (`intel-media-va-driver`, `mesa-va-drivers`) |
| NVIDIA | `nvh264enc` | plugins-bad con NVENC y controlador propietario |
| ARM (Raspberry Pi y similares) | `v4l2h264enc` | plugins-good |
| Respaldo por software | `openh264enc`, `x264enc` | plugins-bad (openh264), plugins-ugly (x264) |

Para ver cuáles hay: `gst-inspect-1.0 | grep -i h264enc`.

## Receptores

| Estado | Receptores |
| --- | --- |
| Probado | Televisores Samsung (Tizen, 2020) |
| Probables | LG (webOS), Roku, Amazon Fire TV, Windows con la aplicación "Conectar" (Pantalla inalámbrica) |

Si prueba otro receptor, abra un *issue* con la marca, el modelo y el resultado.

## Instalación

### 1. Dependencias

Ubuntu y Debian (se recomienda simular primero con `-s` y revisar que no elimine paquetes):

```bash
sudo apt install build-essential pkg-config clang \
  qt6-base-dev qt6-base-private-dev qt6-declarative-dev \
  qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-layouts \
  qml6-module-qtquick-templates qml6-module-qtquick-window qml6-module-qtqml-workerscript \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-tools \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-pipewire \
  network-manager xdg-desktop-portal iw
```

Fedora (los plugins ugly y libav completos vienen de RPM Fusión):

```bash
sudo dnf install gcc-c++ clang pkgconf-pkg-config \
  qt6-qtbase-devel qt6-qtbase-private-devel qt6-qtdeclarative-devel \
  gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-good \
  gstreamer1-plugins-bad-free gstreamer1-plugins-ugly gstreamer1-plugin-openh264 \
  gstreamer1-plugin-libav pipewire-gstreamer NetworkManager xdg-desktop-portal iw
```

Arch Linux:

```bash
sudo pacman -S --needed base-devel clang qt6-base qt6-declarative \
  gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly \
  gst-libav gst-plugin-pipewire gst-plugin-va networkmanager xdg-desktop-portal iw
```

Rust 1.85 o superior con [rustup](https://rustup.rs).

### 2. Compilar e instalar

```bash
git clone https://github.com/andresgarcia0313/proyectar.git
cd proyectar
empaque/instalar.sh            # binario en ~/.local/bin, icono y lanzador en el menú
empaque/instalar.sh --sistema  # una vez, con sudo: nombre del equipo en la TV y cortafuegos
empaque/instalar.sh --quitar   # desinstalar
```

El instalador no pide `sudo`: todo queda en la carpeta del usuario. La opción `--sistema` es aparte
porque NetworkManager no fija el nombre Wi-Fi Direct: sin ella wpa_supplicant anuncia un nombre vacío
y algunos televisores muestran "0" en el aviso de permitir. Instala un script de
NetworkManager-dispatcher que anuncia el nombre del equipo y el tipo "Computador", como Windows. Las opciones y las pantallas
recientes se guardan en `~/.config/proyectar/recientes.json`.

### 3. Cortafuegos

La TV abre una conexión **hacia** el equipo (control RTSP en el TCP 7236) y, cuando el equipo queda como
propietario del grupo Wi-Fi Direct, pide dirección por DHCP. Abra esos puertos solo en las interfaces
del grupo, que NetworkManager nombra `p2p-*`.

Con ufw:

```bash
sudo ufw allow in on p2p-+ to any port 7236 proto tcp
sudo ufw allow in on p2p-+ to any port 67 proto udp
sudo ufw allow in on p2p-+ to any port 68 proto udp
sudo ufw allow in on p2p-+ to any port 5001 proto udp   # informes RTCP del televisor
```

Con firewalld:

```bash
sudo firewall-cmd --permanent --new-zone=proyectar
sudo firewall-cmd --permanent --zone=proyectar --add-interface=p2p-+
sudo firewall-cmd --permanent --zone=proyectar --add-port=7236/tcp
sudo firewall-cmd --permanent --zone=proyectar --add-port=67/udp
sudo firewall-cmd --permanent --zone=proyectar --add-port=68/udp
sudo firewall-cmd --permanent --zone=proyectar --add-port=5001/udp
sudo firewall-cmd --reload
```

## Uso

1. En la TV, abra la fuente "Screen Mirroring" (o el equivalente de su marca).
2. Abra Proyectar desde el menú de aplicaciones y elija la TV en la lista.
3. Acepte la conexión en la TV con el control remoto (solo la primera vez: la TV la recuerda).

Mientras conecta, Proyectar muestra en qué paso va: invitar, enlazar, abrir el video y proyectar.

### Latencia

La tarjeta muestra "Equipo N ms": el retardo dentro del computador, de la captura a la red. El
televisor suma su propio búfer, y es la parte que más pesa. El modo **Juegos** ataca esa parte:

- Audio LPCM sin comprimir si el televisor lo ofrece: no hay retardo de codificación y el TV espera
  menos para sincronizar voz e imagen.
- Refresco intra (x264, escalado en la GPU): una franja se renueva en cada cuadro en lugar de enviar
  cuadros clave grandes, así el búfer del decodificador baja a unos 3 cuadros (50 ms a 60 fps). Si el
  equipo está tan ocupado que no sostiene el 80 % de los fps durante 5 s, sigue con la GPU.
- Envío a ritmo constante: los paquetes salen repartidos, sin ráfagas que agranden el búfer del TV.
- Informes RTCP del televisor (pérdida y fluctuación) que ajustan el bitrate y quedan en el registro.

El botón del cronómetro, en la tarjeta, abre un reloj en milisegundos: llévalo a la pantalla que se
proyecta, toma una foto donde se vean el portátil y el TV y resta los dos tiempos.

### Monitor de comportamiento

Proyectar se evalúa solo mientras se usa. Al arrancar anota un autodiagnóstico del equipo (Wi-Fi
Direct, codificadores, protocolo de KWin, audio, puertos). Durante la proyección registra el retardo
del equipo, el tiempo de ida y vuelta por RTCP, la pérdida, los cuadros descartados, los cuadros clave
que pide el TV, el nivel de audio, la señal Wi-Fi, la CPU y la carga. Al terminar cada sesión deja
un resumen con un veredicto por prueba (latencia, imagen, audio, Extender, Duplicar, estabilidad,
enlace, reconexión): bien, aviso, mal o sin datos.

```bash
proyectar --informe      # las últimas 5 sesiones con sus veredictos
proyectar --informe 20   # las últimas 20
```

Lo que el equipo no puede observar (la imagen en la pantalla del TV, el sonido que se oye, los
ajustes del TV y la latencia de vidrio a vidrio) queda "sin datos"; la latencia completa se obtiene
con el cronómetro.

### Si algo falla

Proyectar explica en qué paso se detuvo y qué revisar (por ejemplo, "El televisor se enlazó pero no
abrió el video"). Cada paso queda anotado, una línea JSON por evento, en
`~/.local/state/proyectar/eventos.jsonl`, que se abre desde el botón "Ver registro":

```bash
jq -c 'select(.evento | test("fallo|p2p_estado|enlace|tv_"))' ~/.local/state/proyectar/eventos.jsonl
```

`p2p_estado` trae el código de motivo de NetworkManager. `journalctl -u NetworkManager -u wpa_supplicant`
completa la historia del lado del sistema.

### Modo demo

Para revisar la interfaz sin TV ni Wi-Fi Direct:

```bash
PROYECTAR_DEMO=1 proyectar
```

También acepta una escena concreta: `lista`, `conectando`, `proyectando`, `reconectando`, `error` o
`vacia`.

## Desarrollo

| Ruta | Contenido |
| --- | --- |
| `crates/wfd` | Protocolo puro: RTSP, negociación M1 a M7, keep-alive, elementos de información |
| `crates/motor` | Wi-Fi Direct (NetworkManager por D-Bus), captura (portal), medios (GStreamer), latencia y recuperación |
| `crates/app` | Puente cxx-qt entre el motor y la interfaz QML |
| `qml/Proyectar` | Componentes QML con tema Drácula |
| `empaque/` | Instalador y lanzador del menú |
| `docs/` | Análisis, diseño, interfaz, calidad, despliegue y gestión |

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test -p wfd
cargo test -p motor --lib
```

| Área | Documento |
| --- | --- |
| Análisis | [docs/01-analisis/requisitos.md](docs/01-analisis/requisitos.md) |
| Diseño | [docs/02-diseno/arquitectura.md](docs/02-diseno/arquitectura.md) |
| Interfaz | [docs/03-ui-ux/especificacion-ui-ux.md](docs/03-ui-ux/especificacion-ui-ux.md) |
| Calidad | [docs/04-calidad/plan-pruebas.md](docs/04-calidad/plan-pruebas.md) |
| Despliegue | [docs/05-despliegue/instalacion.md](docs/05-despliegue/instalacion.md) |
| Gestión | [docs/06-gestion/iteracion-0.1.0.md](docs/06-gestion/iteracion-0.1.0.md) |

## Licencia

GPL-3.0-or-later. Texto completo en [LICENSE](LICENSE). Autor: Andrés García.

---

## English

**Proyectar** ("to project" in Spanish) streams your screen and audio wirelessly to **Miracast**
(Wi-Fi Display) TVs from Linux, as simply as Win+K on Windows. Rust core, Qt 6 / QML interface with a
Dracula palette. The interface text is in Spanish.

### What it is

A Miracast **source**: it discovers nearby TVs over Wi-Fi Direct, links to the chosen one through
NetworkManager, negotiates Wi-Fi Display over RTSP and sends the screen as GPU-encoded H.264 plus audio in
MPEG-TS over RTP. Your regular Wi-Fi stays connected while projecting.

### Features

- **Discovery:** lists nearby Miracast displays with name and signal strength, no setup needed.
- **Three ways to project:** Duplicar (mirror), Extender (extend, creates a virtual monitor for the TV)
  and Ventana (a single window).
- **Latency modes:** Juegos / Games (720p60, lowest latency), Trabajo / Work (1080p30) and
  Videos (1080p60).
- **Audio to the TV:** system sound plays on the TV; when disabled, silence is sent so the TV keeps the
  session.
- **Recent displays:** remembers the last displays and the capture permission for one-click connections.
- **Automatic reconnection:** recovers from link, capture or encoder failures, falling back to the next
  encoder in the chain without dropping the session.

Not included: HDCP, input from the TV (UIBC) and Miracast over Infrastructure (MICE).

### Requirements

- Linux with NetworkManager 1.16 or newer (first release with `wifi-p2p` devices).
- A Wi-Fi card with P2P support: `iw list | grep -A8 "valid interface combinations"` must list `P2P-GO`.
- PipeWire and xdg-desktop-portal (Extender needs a portal with virtual sources, such as KDE Plasma 6).
- GStreamer 1.22 or newer with the base, good, bad, ugly and libav plugins.
- Qt 6 (6.4 or newer) and, to build, Rust 1.85 or newer.

### GPUs and encoders

Proyectar picks the first available H.264 encoder and falls back to the next one on failure:
Intel and AMD through VA-API (`vah264lpenc`, `vah264enc`), NVIDIA (`nvh264enc`), ARM (`v4l2h264enc`) and
software fallback (`openh264enc`, `x264enc`). Check yours with `gst-inspect-1.0 | grep -i h264enc`.

### Receivers

Tested: Samsung TVs (Tizen, 2020). Likely to work: LG (webOS), Roku, Amazon Fire TV and Windows with the
"Connect" app (Wireless Display). Reports about other receivers are welcome as issues.

### Installation

Install the dependencies for your distribution (the apt, dnf and pacman commands are in the Spanish
section above), then:

```bash
git clone https://github.com/andresgarcia0313/proyectar.git
cd proyectar
empaque/instalar.sh            # binary in ~/.local/bin, icon and menu entry
empaque/instalar.sh --quitar   # uninstall
```

No `sudo` is needed. Settings and recent displays live in `~/.config/proyectar/recientes.json`.

### Firewall

The TV connects **to** your computer (RTSP control on TCP 7236) and requests an address over DHCP when
your computer is the Wi-Fi Direct group owner. Open those ports only on the `p2p-*` group interfaces:

```bash
# ufw
sudo ufw allow in on p2p-+ to any port 7236 proto tcp
sudo ufw allow in on p2p-+ to any port 67 proto udp
sudo ufw allow in on p2p-+ to any port 68 proto udp
sudo ufw allow in on p2p-+ to any port 5001 proto udp   # informes RTCP del televisor
```

For firewalld, create a `proyectar` zone bound to `p2p-+` with ports `7236/tcp`, `67/udp` and `68/udp`
(commands in the Spanish section above).

### Troubleshooting

Run `empaque/instalar.sh --sistema` once so TVs show this computer's name instead of "0". While
connecting, Proyectar shows the current step (invite, link, open video, cast); on failure it explains
which step broke and what to check. Every event is logged as JSON lines in
`~/.local/state/proyectar/eventos.jsonl`.

### Usage and demo mode

Set the TV to "Screen Mirroring", open Proyectar from the application menu, pick the TV and accept the
connection on the TV. To preview the interface without a TV:

```bash
PROYECTAR_DEMO=1 proyectar
```

Specific scenes: `lista`, `conectando`, `proyectando`, `reconectando`, `error`, `vacia`.

### License

GPL-3.0-or-later. See [LICENSE](LICENSE). Author: Andrés García.
