import QtQuick
import QtQuick.Layouts

// While inviting, after 10 s without news: did the TV ask to allow the connection?
// "No apareció" reveals how to turn that prompt on; "Sí, ya acepté" puts the card away.
Rectangle {
    id: pregunta
    property bool invitando: false
    property int espera: 10000
    property bool vencida: false
    property string respuesta: ""
    visible: invitando && vencida && respuesta !== "si"
    implicitHeight: col.implicitHeight + Theme.s3 * 2
    radius: Theme.radiusControl
    color: Theme.tint(Theme.accent, 0.10)
    border.color: Theme.tint(Theme.accent, 0.35)
    Accessible.role: Accessible.Grouping
    Accessible.name: titulo.text

    onInvitandoChanged: { vencida = false; respuesta = "" }
    Timer {
        interval: pregunta.espera
        running: pregunta.invitando && !pregunta.vencida
        onTriggered: pregunta.vencida = true
    }

    ColumnLayout {
        id: col
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.s3
        spacing: Theme.s2
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.s2
            Icon { glyph: "circle-help"; color: Theme.accent; Layout.alignment: Qt.AlignTop }
            Text {
                id: titulo
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: "¿Apareció «Permitir» en el televisor?"
                color: Theme.text
                wrapMode: Text.WordWrap
                font.pointSize: Theme.body
                font.weight: Font.DemiBold
            }
        }
        ListaPasos {
            Layout.fillWidth: true
            visible: pregunta.respuesta === "no"
            pasos: [
                "En Samsung abre Configuración > General > Administrador de dispositivos externos > Administrador de conexión de dispositivos > Notificación de acceso y actívala.",
                "Con el control abre Fuente > Screen Mirroring y espera el aviso para aceptarlo."
            ]
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.s2
            PrimaryButton {
                Layout.fillWidth: true
                Layout.preferredWidth: 1
                text: "Sí, ya acepté"
                fill: Theme.selected
                labelColor: Theme.text
                onClicked: pregunta.respuesta = "si"
            }
            PrimaryButton {
                Layout.fillWidth: true
                Layout.preferredWidth: 1
                visible: pregunta.respuesta === ""
                text: "No apareció"
                fill: Theme.selected
                labelColor: Theme.text
                onClicked: pregunta.respuesta = "no"
            }
        }
    }
}
