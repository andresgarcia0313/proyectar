import QtQuick

// One quiet line of numbers: resolution, frames per second, bitrate and which encoder.
Text {
    textFormat: Text.PlainText
    id: line
    property string resolucion
    property real fps: 0
    property int kbps: 0
    property bool gpu: false
    readonly property string velocidad: (kbps / 1000).toFixed(1).replace(".", ",") + " Mb/s"
    text: [resolucion, Math.round(fps) + " fps", velocidad,
           gpu ? "Codificador GPU" : "Codificador por software"].join(" · ")
    color: Theme.textMuted
    font.pointSize: Theme.caption
    wrapMode: Text.WordWrap
    Accessible.role: Accessible.StaticText
    Accessible.name: "Estadísticas: " + text
}
