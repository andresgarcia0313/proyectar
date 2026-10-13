import QtQuick
import QtQuick.Layouts

// Numbered instructions, one per line, read in order.
ColumnLayout {
    id: lista
    property var pasos: []
    property color tono: Theme.accent
    spacing: Theme.s2

    Repeater {
        model: lista.pasos
        delegate: RowLayout {
            required property int index
            required property string modelData
            Layout.fillWidth: true
            spacing: Theme.s2
            Accessible.role: Accessible.StaticText
            Accessible.name: "Paso " + (index + 1) + ": " + modelData
            Rectangle {
                Layout.alignment: Qt.AlignTop
                Layout.preferredWidth: 20; Layout.preferredHeight: 20
                radius: 10
                color: Theme.tint(lista.tono, 0.18)
                Text {
                    textFormat: Text.PlainText
                    anchors.centerIn: parent
                    text: index + 1
                    color: lista.tono
                    font.pointSize: Theme.caption
                    font.weight: Font.DemiBold
                }
            }
            Text {
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: modelData
                color: Theme.text
                wrapMode: Text.WordWrap
                font.pointSize: Theme.label
            }
        }
    }
}
