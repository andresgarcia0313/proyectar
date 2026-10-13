import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Segmented control: equal segments in a track, the chosen one raised. One choice only.
Rectangle {
    id: control
    property string label
    property var options: []
    property int current: 0
    signal picked(int index)
    implicitHeight: Theme.hit
    implicitWidth: 240
    radius: Theme.radiusControl
    color: Theme.base
    border.color: Theme.selected
    Accessible.role: Accessible.Grouping
    Accessible.name: label

    RowLayout {
        anchors.fill: parent
        anchors.margins: 2
        spacing: 2
        Repeater {
            model: control.options
            delegate: AbstractButton {
                id: seg
                required property int index
                required property string modelData
                readonly property bool chosen: index === control.current
                Layout.fillWidth: true
                Layout.fillHeight: true
                hoverEnabled: true
                focusPolicy: Qt.StrongFocus
                checkable: true
                checked: chosen
                text: modelData
                Accessible.name: modelData
                Accessible.role: Accessible.RadioButton
                onClicked: control.picked(index)
                contentItem: Text {
                    text: seg.text
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideRight
                    font.pointSize: Theme.label
                    font.weight: seg.chosen ? Font.DemiBold : Font.Normal
                    color: seg.chosen ? Theme.text : Theme.textMuted
                }
                background: Rectangle {
                    radius: Theme.radiusControl - 2
                    color: seg.chosen ? Theme.selected : seg.hovered ? Theme.hover : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.fast } }
                    FocusRing { target: seg }
                }
            }
        }
    }
}
