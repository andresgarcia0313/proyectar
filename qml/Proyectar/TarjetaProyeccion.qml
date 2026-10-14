import QtQuick
import QtQuick.Layouts

// The projection in progress: where it goes, which step it is on, and the way out.
Rectangle {
    id: card
    property int codigo: 0
    property string nombre
    property string texto
    property bool hayEstadisticas: false
    property string resolucion
    property real fps: 0
    property int kbps: 0
    property bool gpu: false
    property real retardo: 0
    // The `Proyeccion` object, where the stopwatch logs the delay the user measured.
    property var registro: null
    signal desconectar()
    readonly property bool proyectando: codigo === 4
    readonly property color tono: proyectando ? Theme.ok : codigo === 5 ? Theme.warn : Theme.accent
    radius: Theme.radiusCard
    color: Theme.raised
    implicitHeight: col.implicitHeight + Theme.s5 * 2

    ColumnLayout {
        id: col
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.s5
        spacing: Theme.s3
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: 64; Layout.preferredHeight: 64
            // Room for the question card: the step indicator already says what is going on.
            visible: !pregunta.visible
            radius: 32
            color: Theme.tint(card.tono, 0.16)
            Icon { anchors.centerIn: parent; glyph: "tv"; size: 30; color: card.tono }
        }
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: card.texto
            color: Theme.text
            font.pointSize: Theme.title * 0.85
            font.weight: Font.DemiBold
            Accessible.role: Accessible.StaticText
            Accessible.name: card.texto
        }
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            visible: card.codigo === 3 || card.codigo === 5
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideRight
            text: card.nombre
            color: Theme.textMuted
            font.pointSize: Theme.label
        }
        PasosConexion {
            Layout.fillWidth: true
            Layout.topMargin: Theme.s1
            codigo: card.codigo
            tono: card.tono
        }
        PreguntaPermitir {
            id: pregunta
            Layout.fillWidth: true
            invitando: card.codigo === 2
        }
        Estadisticas {
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            visible: card.proyectando && card.hayEstadisticas
            resolucion: card.resolucion; fps: card.fps; kbps: card.kbps; gpu: card.gpu; retardo: card.retardo
        }
        PrimaryButton {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: Theme.s2
            text: card.proyectando || card.codigo === 5 ? "Desconectar" : "Cancelar"
            Layout.fillWidth: true
            fill: Theme.selected
            labelColor: Theme.text
            onClicked: card.desconectar()
        }
    }
    // Discreet, in the corner: the card keeps its height in the smallest window.
    IconButton {
        anchors { top: parent.top; right: parent.right; margins: Theme.s2 }
        visible: card.proyectando
        iconName: "timer"; tint: Theme.accent
        tip: "Medir latencia"
        Accessible.description: "Abre un cronómetro para fotografiar el portátil y el TV a la vez"
        onClicked: { cronometro.show(); cronometro.raise(); cronometro.requestActivate() }
    }
    Cronometro { id: cronometro; registro: card.registro }
}
