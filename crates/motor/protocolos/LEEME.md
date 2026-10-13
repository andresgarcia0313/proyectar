<!-- lang: es-CO -->
# Protocolos de Wayland de KDE

Copias sin cambios de [plasma-wayland-protocols](https://invent.kde.org/libraries/plasma-wayland-protocols)
(KDE Frameworks 6.11), para compilar sin depender del paquete del sistema.

| Archivo | Uso | Licencia |
|---|---|---|
| `zkde-screencast-unstable-v1.xml` | Captura directa de pantallas y salidas virtuales de KWin | `LICENSES/LGPL-2.1-or-later.txt` |
| `kde-output-order-v1.xml` | Orden de las pantallas: la primera es la principal | `LICENSES/MIT-CMU.txt` |

KWin solo ofrece `zkde_screencast_unstable_v1` a los programas cuyo lanzador `.desktop` lo declara
en `X-KDE-Wayland-Interfaces` y cuyo `Exec` apunta al mismo ejecutable.

## Control de cambios

| Fecha | Cambio |
|---|---|
| 2026-10-13 | Se agregan los dos protocolos y sus licencias |
