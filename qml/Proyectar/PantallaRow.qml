import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// One display in the list: name, whether it is a recent one, and how well it is heard.
ItemDelegate {
    id: row
    property string nombre
    property int senal: 0
    property bool cercana: true
    property bool reciente: false
    property bool elegida: false
    signal activada()
    readonly property string calidad: !cercana ? "Fuera de alcance"
        : senal >= 75 ? "Señal excelente" : senal >= 50 ? "Señal buena"
        : senal >= 25 ? "Señal regular" : "Señal débil"

    implicitHeight: 56
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    opacity: cercana ? 1 : 0.6
    Accessible.name: nombre + ", " + (reciente ? "usada antes, " : "") + calidad
    Accessible.role: Accessible.ListItem
    Accessible.selected: elegida
    onDoubleClicked: if (cercana) row.activada()

    contentItem: RowLayout {
        spacing: Theme.s3
        Rectangle {
            Layout.preferredWidth: 36; Layout.preferredHeight: 36
            radius: 18
            color: Theme.tint(Theme.accent, row.elegida ? 0.28 : 0.14)
            Icon { anchors.centerIn: parent; glyph: "tv"; color: Theme.accent }
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0
            Text {
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: row.nombre
                elide: Text.ElideRight
                color: Theme.text
                font.pointSize: Theme.body
                font.weight: Font.DemiBold
            }
            RowLayout {
                spacing: Theme.s1
                Icon { glyph: "star"; size: 12; color: Theme.warn; visible: row.reciente }
                Text {
                    textFormat: Text.PlainText
                    text: (row.reciente ? "Usada antes · " : "") + row.calidad
                    color: Theme.textMuted
                    font.pointSize: Theme.caption
                }
            }
        }
        SenalBars { level: row.cercana ? row.senal : 0; Layout.alignment: Qt.AlignVCenter }
    }
    background: Rectangle {
        radius: Theme.radiusControl
        color: row.elegida ? Theme.selected : row.hovered ? Theme.hover : "transparent"
        Behavior on color { ColorAnimation { duration: Theme.fast } }
        FocusRing { target: row }
    }
}
