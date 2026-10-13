import QtQuick

// Where the connection is: Invitar, Enlazar, Abrir video, Proyectando. Finished steps carry
// a check, the current one breathes (still when reduce motion is on), the rest wait in grey.
Item {
    id: pasos
    property int codigo: 0
    property color tono: Theme.accent
    readonly property var nombres: ["Invitar", "Enlazar", "Abrir video", "Proyectando"]
    // Engine codes (avisos.rs) to step: 2 inviting, 3 linked, 6 RTSP, 4 projecting, 5 again.
    readonly property int actual: [0, 0, 0, 1, 3, 0, 2][codigo] || 0
    readonly property bool terminado: codigo === 4
    readonly property real celda: width / nombres.length
    implicitHeight: 24 + Theme.s2 + etiquetaAlto
    property real etiquetaAlto: 16
    Accessible.role: Accessible.ProgressBar
    Accessible.name: terminado ? "Proyectando"
        : "Paso " + (actual + 1) + " de 4: " + nombres[actual]

    Repeater {
        model: pasos.nombres
        delegate: Item {
            id: paso
            required property int index
            required property string modelData
            readonly property bool hecho: index < pasos.actual || pasos.terminado
            readonly property bool enCurso: index === pasos.actual && !pasos.terminado
            x: index * pasos.celda
            width: pasos.celda
            height: pasos.height

            Rectangle {
                visible: paso.index < pasos.nombres.length - 1
                x: paso.width / 2; y: 11
                width: paso.width; height: 2
                color: paso.index < pasos.actual || pasos.terminado ? pasos.tono : Theme.selected
                Behavior on color { ColorAnimation { duration: Theme.normal } }
            }
            Rectangle {
                id: circulo
                anchors.horizontalCenter: parent.horizontalCenter
                width: 24; height: 24; radius: 12
                color: paso.hecho ? pasos.tono : Theme.raised
                border.width: paso.hecho ? 0 : 2
                border.color: paso.enCurso ? pasos.tono : Theme.border
                Behavior on color { ColorAnimation { duration: Theme.normal } }
                Icon {
                    anchors.centerIn: parent
                    visible: paso.hecho
                    glyph: "check"; size: 14; color: Theme.accentLabel
                }
                Rectangle {
                    anchors.centerIn: parent
                    visible: paso.enCurso
                    width: 10; height: 10; radius: 5
                    color: pasos.tono
                    SequentialAnimation on opacity {
                        running: paso.enCurso && !Theme.reduceMotion
                        loops: Animation.Infinite
                        NumberAnimation { to: 0.3; duration: Theme.slow; easing.type: Easing.InOutQuad }
                        NumberAnimation { to: 1; duration: Theme.slow; easing.type: Easing.InOutQuad }
                    }
                }
            }
            Text {
                textFormat: Text.PlainText
                anchors.top: circulo.bottom
                anchors.topMargin: Theme.s2
                width: paso.width
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
                text: paso.modelData
                color: paso.enCurso || paso.hecho ? Theme.text : Theme.textMuted
                font.pointSize: Theme.caption
                font.weight: paso.enCurso ? Font.DemiBold : Font.Normal
                onImplicitHeightChanged: pasos.etiquetaAlto = implicitHeight
            }
        }
    }
}
