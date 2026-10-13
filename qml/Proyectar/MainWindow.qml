import QtQuick
import QtQuick.Layouts

// The Win+K panel: nearby displays, one Connect button, then the projection card.
Window {
    id: win
    // The `Proyeccion` object from Rust (see src/proyeccion_object.rs).
    required property var proy
    // Size remembered between runs and the version for the footer (src/ventana.rs).
    required property var ventana
    property string seleccion
    readonly property bool enLista: proy.codigo < 2
    // Inviting, linking or opening the video: only the connection and its way out are shown.
    readonly property bool enCamino: [2, 3, 6].indexOf(proy.codigo) >= 0
    readonly property bool seleccionValida: {
        const f = lista.filas.find(x => x.mac === seleccion)
        return f !== undefined && f.cercana
    }

    title: "Proyectar"
    width: ventana.ancho; height: ventana.alto
    minimumWidth: 340; minimumHeight: 560
    onWidthChanged: guardar.restart()
    onHeightChanged: guardar.restart()
    Timer { id: guardar; interval: 600; onTriggered: win.ventana.recordar(win.width, win.height) }
    color: Theme.base
    visible: true

    Timer { interval: 5000; repeat: true; running: true; onTriggered: win.proy.beat() }
    Component.onCompleted: proy.buscar()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.s4
        spacing: Theme.s3

        RowLayout {
            Layout.fillWidth: true
            Icon { glyph: "cast"; size: 24; color: Theme.accent }
            Text {
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: "Proyectar"
                color: Theme.text
                font.pointSize: Theme.title
                font.weight: Font.DemiBold
                Accessible.role: Accessible.Heading
            }
            IconButton {
                iconName: "rotate-ccw"; tip: "Buscar pantallas de nuevo"
                visible: win.enLista
                onClicked: win.proy.buscar()
            }
        }

        RowLayout {
            Layout.fillWidth: true
            visible: win.enLista && !diagnostico.visible
            spacing: Theme.s2
            Rectangle {
                width: 8; height: 8; radius: 4
                color: win.proy.codigo === 1 ? Theme.accent : Theme.border
            }
            Text {
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: win.proy.texto
                color: Theme.textMuted
                font.pointSize: Theme.label
                Accessible.role: Accessible.StaticText
            }
        }

        TarjetaDiagnostico {
            id: diagnostico
            Layout.fillWidth: true
            // In place of the list: the steps scroll if the window is short.
            Layout.fillHeight: true
            Layout.maximumHeight: implicitHeight
            json: win.proy.diagnostico
            rutaRegistro: win.proy.rutaRegistro
            onReintentar: win.proy.reintentar()
            onDescartar: win.proy.descartarDiagnostico()
        }
        ErrorBanner {
            Layout.fillWidth: true
            message: diagnostico.visible ? "" : win.proy.ultimoError
            onDismissed: win.proy.descartarError()
        }

        PantallaLista {
            id: lista
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: win.enLista && !diagnostico.visible
            pantallasJson: win.proy.pantallas
            buscando: win.proy.codigo === 1
            seleccion: win.seleccion
            onElegida: (mac) => win.seleccion = mac
            onConectarPedido: (mac) => { win.seleccion = mac; win.proy.conectar(mac) }
        }
        PrimaryButton {
            Layout.fillWidth: true
            visible: win.enLista && !diagnostico.visible
            enabled: win.seleccionValida
            fill: enabled ? Theme.accent : Theme.selected
            labelColor: enabled ? Theme.accentLabel : Theme.textMuted
            text: "Conectar"
            iconName: "cast"
            onClicked: win.proy.conectar(win.seleccion)
        }

        TarjetaProyeccion {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !win.enLista
            codigo: win.proy.codigo
            nombre: win.proy.nombre
            texto: win.proy.texto
            hayEstadisticas: win.proy.hayEstadisticas
            resolucion: win.proy.resolucion
            fps: win.proy.fps
            kbps: win.proy.kbps
            gpu: win.proy.gpu
            retardo: win.proy.retardo
            onDesconectar: win.proy.desconectar()
        }

        Opciones {
            Layout.fillWidth: true
            visible: !win.enCamino
            fuente: win.proy.fuente
            latencia: win.proy.latencia
            audio: win.proy.audio
            onFuenteElegida: (i) => win.proy.elegirFuente(i)
            onLatenciaElegida: (i) => win.proy.elegirLatencia(i)
            onAudioElegido: (activo) => win.proy.elegirAudio(activo)
        }

        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            text: "Proyectar " + win.ventana.version
            color: Theme.textMuted
            font.pointSize: Theme.caption
            opacity: 0.7
        }
    }
}
