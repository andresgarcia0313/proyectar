import QtQuick
import QtQuick.Layouts

// The last failure in plain Spanish; stays until dismissed or the next attempt.
Rectangle {
    id: banner
    property string message
    signal dismissed()
    visible: message !== ""
    implicitHeight: row.implicitHeight + Theme.s3 * 2
    radius: Theme.radiusControl
    color: Theme.tint(Theme.danger, 0.14)
    border.color: Theme.tint(Theme.danger, 0.5)
    Accessible.role: Accessible.AlertMessage
    Accessible.name: message

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: Theme.s3
        spacing: Theme.s2
        Icon { glyph: "circle-alert"; color: Theme.danger; Layout.alignment: Qt.AlignTop }
        Text {
            Layout.fillWidth: true
            text: banner.message
            color: Theme.text
            wrapMode: Text.WordWrap
            font.pointSize: Theme.label
        }
        IconButton {
            iconName: "x"; tip: "Descartar aviso"
            Layout.alignment: Qt.AlignTop
            implicitWidth: Theme.hit; implicitHeight: Theme.hit
            onClicked: banner.dismissed()
        }
    }
}
