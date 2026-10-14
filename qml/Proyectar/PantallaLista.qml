import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Nearby displays, recent ones first. Click chooses; double click or Enter connects.
Rectangle {
    id: lista
    property string pantallasJson: "[]"
    property bool buscando: false
    property string seleccion
    signal elegida(string mac)
    signal conectarPedido(string mac)
    readonly property var filas: JSON.parse(pantallasJson || "[]")
    radius: Theme.radiusCard
    color: Theme.raised
    Accessible.role: Accessible.List
    Accessible.name: "Pantallas cercanas"

    ListView {
        id: vista
        anchors.fill: parent
        anchors.margins: Theme.s2
        model: lista.filas
        spacing: 2
        clip: true
        keyNavigationEnabled: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar {}
        delegate: PantallaRow {
            required property var modelData
            width: vista.width
            nombre: modelData.nombre
            senal: modelData.senal
            cercana: modelData.cercana
            reciente: modelData.reciente
            elegida: modelData.mac === lista.seleccion
            onClicked: if (cercana) lista.elegida(modelData.mac)
            onActivada: lista.conectarPedido(modelData.mac)
            Keys.onReturnPressed: if (cercana) lista.conectarPedido(modelData.mac)
            Keys.onEnterPressed: if (cercana) lista.conectarPedido(modelData.mac)
        }
    }
    ColumnLayout {
        anchors.centerIn: parent
        width: parent.width - Theme.s5 * 2
        visible: lista.filas.length === 0
        spacing: Theme.s2
        Icon { glyph: "cast"; size: 32; color: Theme.border; Layout.alignment: Qt.AlignHCenter }
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            color: Theme.text
            font.pointSize: Theme.body
            font.weight: Font.DemiBold
            text: lista.buscando ? "Buscando pantallas…" : "No hay pantallas cerca"
        }
        Text {
            textFormat: Text.PlainText
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pointSize: Theme.label
            text: "Enciende el televisor y activa la duplicación de pantalla (Screen Mirroring)."
                  + (lista.buscando ? "" : " Luego toca ↻ para buscar de nuevo.")
        }
    }
}
