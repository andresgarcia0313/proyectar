pragma Singleton
import QtQuick

// Design tokens, Dracula only (a wireless-display panel is used in dim rooms).
QtObject {
    property bool reduceMotion: false

    readonly property color base: "#282A36"
    readonly property color raised: "#343746"
    readonly property color hover: "#424450"
    readonly property color selected: "#44475A"
    readonly property color text: "#F8F8F2"
    readonly property color textMuted: "#B6BCD8"
    readonly property color border: "#6272A4"
    readonly property color accent: "#BD93F9"
    readonly property color accentLabel: "#282A36"
    readonly property color ok: "#50FA7B"
    readonly property color warn: "#FFB86C"
    readonly property color danger: "#FF5555"

    function tint(color, alpha) { return Qt.rgba(color.r, color.g, color.b, alpha) }

    // Type scale relative to the system font so the OS text size setting is respected.
    property real basePt: Qt.application.font.pointSize > 0
        ? Qt.application.font.pointSize : 10
    readonly property real title: basePt * 1.45
    readonly property real body: basePt
    readonly property real label: basePt * 0.92
    readonly property real caption: Math.max(9, basePt * 0.85)

    readonly property int s1: 4
    readonly property int s2: 8
    readonly property int s3: 12
    readonly property int s4: 16
    readonly property int s5: 24

    readonly property int radiusControl: 8
    readonly property int radiusCard: 12
    readonly property int hit: 36

    readonly property int fast: reduceMotion ? 0 : 120
    readonly property int normal: reduceMotion ? 0 : 200
    readonly property int slow: reduceMotion ? 0 : 900
}
