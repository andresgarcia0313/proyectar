import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// "Medir latencia": a window of its own with a large stopwatch. Shown on the projected screen
// and photographed with the TV in the same shot, the two readings differ by the real delay.
Window {
    id: crono
    objectName: "cronometro"
    // The `Proyeccion` object: `anotarLatencia(texto)` logs the difference the user read.
    property var registro: null
    property string aviso
    title: "Medir latencia"
    width: 760; height: 520
    minimumWidth: 420; minimumHeight: 340
    color: Theme.base
    onVisibleChanged: if (visible) { aviso = ""; campo.text = "" }

    Shortcut { sequences: [StandardKey.Cancel]; onActivated: crono.close() }
    Shortcut { sequence: "F11"; onActivated: crono.pantallaCompleta() }
    function pantallaCompleta() {
        crono.visibility = crono.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen
    }
    function guardar() {
        const bien = crono.registro !== null && crono.registro.anotarLatencia(campo.text)
        aviso = bien ? "Guardado en el registro de eventos." : "Escribe la diferencia en milisegundos, por ejemplo 120."
        if (bien) campo.text = ""
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.s5
        spacing: Theme.s4
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Pon esta ventana en la pantalla que se proyecta, toma una foto donde se vean el portátil y el TV y resta los dos tiempos."
            color: Theme.text
            font.pointSize: Theme.body
        }
        RelojLatencia {
            Layout.fillWidth: true
            Layout.fillHeight: true
            corriendo: crono.visible
            MouseArea { anchors.fill: parent; onDoubleClicked: crono.pantallaCompleta() }
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.s2
            TextField {
                id: campo
                Layout.fillWidth: true
                implicitHeight: 44
                placeholderText: "Diferencia medida en ms (opcional)"
                placeholderTextColor: Theme.textMuted
                color: Theme.text
                selectionColor: Theme.accent
                selectedTextColor: Theme.accentLabel
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                font.pointSize: Theme.body
                Accessible.name: "Diferencia medida en milisegundos"
                background: Rectangle {
                    radius: Theme.radiusControl
                    color: Theme.raised
                    border.color: campo.activeFocus ? Theme.accent : Theme.border
                }
                onAccepted: crono.guardar()
            }
            PrimaryButton {
                anchoMinimo: 0
                text: "Guardar"
                enabled: campo.text.trim() !== ""
                fill: enabled ? Theme.accent : Theme.selected
                labelColor: enabled ? Theme.accentLabel : Theme.textMuted
                onClicked: crono.guardar()
            }
        }
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: crono.aviso !== "" ? crono.aviso : "Doble clic o F11: pantalla completa. Esc: cerrar."
            color: Theme.textMuted
            font.pointSize: Theme.caption
            Accessible.role: Accessible.StaticText
        }
    }
}
