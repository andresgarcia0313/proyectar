import QtQuick

// One quiet line of numbers: resolution, frames per second, bitrate and which encoder.
Text {
    textFormat: Text.PlainText
    id: line
    property string resolucion
    property real fps: 0
    property int kbps: 0
    property bool gpu: false
    // Capture to network inside this laptop; the TV adds its own buffer on top.
    property real retardo: 0
    readonly property string velocidad: (kbps / 1000).toFixed(1).replace(".", ",") + " Mb/s"
    text: [resolucion, Math.round(fps) + " fps", velocidad,
           gpu ? "Codificador GPU" : "Codificador por software"]
          .concat(retardo > 0 ? ["Equipo " + Math.round(retardo) + " ms"] : []).join(" · ")
    color: Theme.textMuted
    font.pointSize: Theme.caption
    wrapMode: Text.WordWrap
    Accessible.role: Accessible.StaticText
    Accessible.name: "Estadísticas: " + text
}
