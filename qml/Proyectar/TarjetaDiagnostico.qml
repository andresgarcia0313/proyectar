import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Why the projection failed and what to try, in order, in place of the plain error banner.
// One main action (Reintentar); the steps scroll when the window is short.
Rectangle {
    id: tarjeta
    property string json // motor::Diagnostico: {causa, titulo, detalle, pasos}
    property string rutaRegistro
    signal reintentar()
    signal descartar()
    readonly property var datos: {
        try { return json ? JSON.parse(json) : null } catch (e) { return null }
    }
    visible: datos !== null
    implicitHeight: cabecera.implicitHeight + cuerpo.implicitHeight + acciones.implicitHeight
                    + Theme.s3 * 2 + Theme.s4 * 2
    radius: Theme.radiusCard
    color: Theme.raised
    border.color: Theme.tint(Theme.warn, 0.45)
    Accessible.role: Accessible.AlertMessage
    Accessible.name: datos ? datos.titulo : ""

    function abrirRegistro() {
        const ruta = rutaRegistro.replace(/\\/g, "/").replace(/^\/+/, "")
        Qt.openUrlExternally("file:///" + ruta)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.s4
        spacing: Theme.s3
        RowLayout {
            id: cabecera
            Layout.fillWidth: true
            spacing: Theme.s2
            Icon { glyph: "circle-alert"; color: Theme.warn; Layout.alignment: Qt.AlignTop }
            Text {
                textFormat: Text.PlainText
                Layout.fillWidth: true
                text: tarjeta.datos ? tarjeta.datos.titulo : ""
                color: Theme.text
                wrapMode: Text.WordWrap
                font.pointSize: Theme.body * 1.05
                font.weight: Font.DemiBold
                Accessible.role: Accessible.Heading
            }
            IconButton {
                iconName: "x"; tip: "Descartar diagnóstico"
                Layout.alignment: Qt.AlignTop
                Layout.topMargin: -Theme.s2
                onClicked: tarjeta.descartar()
            }
        }
        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentHeight: cuerpo.implicitHeight
            clip: true; boundsBehavior: Flickable.StopAtBounds
            ScrollIndicator.vertical: ScrollIndicator {}
            ColumnLayout {
                id: cuerpo
                width: parent.width
                spacing: Theme.s3
                Text {
                    textFormat: Text.PlainText
                    Layout.fillWidth: true
                    visible: text !== ""
                    text: tarjeta.datos ? tarjeta.datos.detalle.charAt(0).toUpperCase() + tarjeta.datos.detalle.slice(1) : ""
                    color: Theme.textMuted
                    wrapMode: Text.WordWrap
                    font.pointSize: Theme.caption
                }
                ListaPasos {
                    Layout.fillWidth: true
                    tono: Theme.warn
                    pasos: tarjeta.datos ? tarjeta.datos.pasos : []
                }
            }
        }
        RowLayout {
            id: acciones
            Layout.fillWidth: true
            spacing: Theme.s2
            PrimaryButton {
                Layout.fillWidth: true
                anchoMinimo: 0
                text: "Reintentar"; iconName: "rotate-ccw"
                onClicked: tarjeta.reintentar()
            }
            PrimaryButton {
                visible: tarjeta.rutaRegistro !== ""
                anchoMinimo: 0
                text: "Ver registro"; fill: "transparent"; labelColor: Theme.accent
                onClicked: tarjeta.abrirRegistro()
            }
        }
    }
}
