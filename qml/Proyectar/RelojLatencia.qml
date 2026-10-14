pragma ComponentBehavior: Bound
import QtQuick

// The stopwatch face: minutes, seconds and milliseconds of the wall clock, redrawn on every
// frame the window shows, and a bar that crosses once per second so motion can be seen.
Item {
    id: reloj
    property bool corriendo: true
    property real ahora: Date.now()
    readonly property real fraccion: (ahora % 1000) / 1000
    readonly property string fija: Qt.platform.os === "windows" ? "Consolas"
                                 : Qt.platform.os === "osx" ? "Menlo" : "monospace"

    function dos(n) { return n < 10 ? "0" + n : "" + n }
    function tres(n) { return n < 10 ? "00" + n : n < 100 ? "0" + n : "" + n }
    function marca(ms) {
        const t = Math.floor(ms % 3600000)
        return dos(Math.floor(t / 60000)) + ":" + dos(Math.floor(t / 1000) % 60) + "." + tres(t % 1000)
    }

    // Tied to the frames the window really presents, not to a timer that can drift or bunch up.
    FrameAnimation {
        running: reloj.corriendo
        onTriggered: reloj.ahora = Date.now()
    }

    Text {
        id: cifras
        textFormat: Text.PlainText
        anchors.horizontalCenter: parent.horizontalCenter
        // Centered in the room left above the bar.
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -(pista.height + Theme.s3) / 2
        text: reloj.marca(reloj.ahora)
        color: Theme.text
        font.family: reloj.fija
        font.pixelSize: Math.max(24, Math.min(reloj.width / 5.8, (reloj.height - pista.height) / 1.6))
        font.weight: Font.Bold
        Accessible.role: Accessible.StaticText
        Accessible.name: "Cronómetro en milisegundos"
    }

    Rectangle {
        id: pista
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: Math.max(28, reloj.height * 0.18)
        radius: Theme.radiusControl
        color: Theme.raised
        Repeater {
            model: 9
            Rectangle {
                required property int index
                x: (index + 1) * pista.width / 10 - width / 2
                width: 2; height: pista.height; color: Theme.selected
            }
        }
        Rectangle {
            x: reloj.fraccion * (pista.width - width)
            width: Math.max(12, pista.width / 40); height: pista.height
            radius: Theme.s1
            color: Theme.ok
        }
    }
}
