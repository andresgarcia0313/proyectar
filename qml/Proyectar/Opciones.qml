import QtQuick
import QtQuick.Layouts

// Screen mode, latency and sound. They apply to the next projection and to the current one.
ColumnLayout {
    id: panel
    property int fuente: 0
    property int latencia: 1
    property bool audio: true
    signal fuenteElegida(int index)
    signal latenciaElegida(int index)
    signal audioElegido(bool activo)
    readonly property var notasFuente: ["Muestra lo mismo que tu pantalla.",
        "Usa el televisor como una segunda pantalla.", "Comparte una sola ventana."]
    readonly property var notasLatencia: ["Menor retraso, 720p a 60 fps.",
        "Texto nítido, 1080p a 30 fps.", "Movimiento fluido, 1080p a 60 fps."]
    spacing: Theme.s3

    ColumnLayout {
        spacing: Theme.s1
        Text { text: "Modo de pantalla"; color: Theme.textMuted; font.pointSize: Theme.label
               font.weight: Font.DemiBold; Accessible.role: Accessible.Heading }
        Segmented {
            Layout.fillWidth: true
            label: "Modo de pantalla"
            options: ["Duplicar", "Extender", "Ventana"]
            current: panel.fuente
            onPicked: (i) => panel.fuenteElegida(i)
        }
        Text { text: panel.notasFuente[panel.fuente]; color: Theme.textMuted
               font.pointSize: Theme.caption }
    }
    ColumnLayout {
        spacing: Theme.s1
        Text { text: "Latencia"; color: Theme.textMuted; font.pointSize: Theme.label
               font.weight: Font.DemiBold; Accessible.role: Accessible.Heading }
        Segmented {
            Layout.fillWidth: true
            label: "Latencia"
            options: ["Juegos", "Trabajo", "Videos"]
            current: panel.latencia
            onPicked: (i) => panel.latenciaElegida(i)
        }
        Text { text: panel.notasLatencia[panel.latencia]; color: Theme.textMuted
               font.pointSize: Theme.caption }
    }
    RowLayout {
        Layout.fillWidth: true
        Text {
            Layout.fillWidth: true
            text: "Audio en el televisor"
            color: Theme.text
            font.pointSize: Theme.body
        }
        ThemedSwitch {
            checked: panel.audio
            Accessible.name: "Audio en el televisor"
            onToggled: panel.audioElegido(checked)
        }
    }
}
