---
lang: es-CO
título: Proyectar - Especificación de experiencia e interfaz
versión: 1.0.1
---

# Proyectar - Especificación de experiencia e interfaz

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-12 | Andrés García | Versión inicial: brief, panel tipo Win+K, estados, textos, paleta y criterios |
| 1.0.1 | 2026-10-12 | Andrés García | Revisión para publicar: sin cambios de contenido |

Método: pautas HIG (Human Interface Guidelines, pautas de interfaz humana de Apple) aplicadas a Qt 6 y
Plasma: claridad, deferencia al contenido, profundidad por capas y una sola acción principal por
pantalla. Sistema visual y componentes en el mismo estilo de Foco.

## 1. Brief

Una persona quiere ver su pantalla en la TV en dos clics, sin saber qué es Wi-Fi Direct. Proyectar
muestra **una lista de pantallas y un estado claro**, y se aparta: panel compacto, sin ventanas
emergentes, sin tomar el foco del teclado, sin sonido propio.

**Tesis de diseño:** "La lista es la aplicación". Todo lo demás (modo, fuente, audio) aparece solo
cuando hay una proyección y ocupa una fila.

## 2. Trabajos y frecuencia

| Trabajo | Frecuencia | Ubicación |
| --- | --- | --- |
| Proyectar a la TV de siempre | Varias veces al día | Primera fila de la lista, un clic |
| Dejar de proyectar | Igual que el anterior | Botón "Desconectar" siempre visible |
| Cambiar entre Duplicar y Extender | Rara vez | Selector segmentado bajo el estado |
| Cambiar de modo (Juegos, Trabajo, Videos) | Rara vez | Selector segmentado |
| Silenciar el audio de la TV | Ocasional | Interruptor "Enviar sonido" |

## 3. Estructura (un solo nivel)

```text
Proyectar (ventana de 380 x 560, ancho mínimo 340)
├── Encabezado: título "Proyectar a una pantalla" + estado de búsqueda
├── Lista de pantallas (recientes primero, luego encontradas por señal)
├── Tarjeta de estado (solo con proyección): nombre, 1080p30, kbps, GPU
├── Opciones (solo con proyección): Fuente, Modo, Enviar sonido
└── Pie: "Desconectar" (primario cuando proyecta)
```

Atajos: Ctrl+R vuelve a buscar, Esc desconecta tras confirmar solo si hay proyección, Enter conecta con
la fila resaltada. Navegación completa con teclado y nombres accesibles en cada fila.

## 4. Estados (derivados de `Estado` del motor)

| Estado | Qué ve la persona | Texto principal |
| --- | --- | --- |
| Inactivo | Lista vacía con icono tenue | "No hay pantallas cerca" y ayuda: "En la TV, abra Source - Screen Mirroring" |
| Buscando | Barra fina indeterminada bajo el encabezado | "Buscando pantallas..." |
| Conectando | Fila con indicador de actividad | "Conectando con {nombre}" |
| EsperandoPantalla | Fila resaltada, aviso | "Acepte la conexión en la TV" |
| Proyectando | Tarjeta de estado y opciones | "Proyectando a {nombre}" |
| Reconectando | Fila ámbar con intento | "Recuperando la conexión (intento {n})" |
| Error | Banda inferior con causa y paso siguiente | Por ejemplo "La TV rechazó la conexión. Vuelva a elegirla en Source" |

Reglas: el estado nunca se comunica solo con color; siempre lleva texto e icono. Los errores dicen qué
pasó y qué hacer, sin términos de red (nada de P2P, RTSP ni DHCP).

## 5. Paleta Drácula

| Rol | Valor | Uso |
| --- | --- | --- |
| `base` | `#282A36` | Fondo |
| `raised` | `#343746` | Tarjetas y filas |
| `hover` | `#424450` | Fila bajo el puntero |
| `selected` | `#44475A` | Fila seleccionada |
| `text` | `#F8F8F2` | Texto principal (contraste 13,36:1) |
| `textMuted` | `#B6BCD8` | Texto secundario (7,58:1) |
| `border` | `#6272A4` | Separadores, no texto |
| `accent` | `#BD93F9` | Acción principal y foco de teclado |
| `ok` | `#50FA7B` | Proyectando |
| `warn` | `#FFB86C` | Reconectando, aceptar en la TV |
| `danger` | `#FF5555` | Errores |

Tipografía del sistema; tamaños 12, 14 y 20 pt. Esquinas de 12 px, espacios múltiplos de 8, filas de
56 px (zona de toque holgada), anillo de foco de 2 px en `accent`.

## 6. Criterios de aceptación

| Id | Criterio | Cómo se comprueba |
| --- | --- | --- |
| UX-01 | De abrir la app a proyectar: máximo 2 clics con una TV reciente | Prueba manual cronometrada |
| UX-02 | Cada estado de la sección 4 tiene texto, icono y no depende del color | Revisión de capturas |
| UX-03 | Contraste de texto de 4,5:1 o más sobre su fondo | Cálculo sobre la paleta |
| UX-04 | Todo se opera con teclado y la ventana funciona a 340 px de ancho | Prueba manual |
| UX-05 | Abrir y proyectar no roba el foco del teclado de otra ventana | Prueba manual en Wayland |
| UX-06 | Ningún mensaje contiene siglas técnicas | Revisión de textos |
