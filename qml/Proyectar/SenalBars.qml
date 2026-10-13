import QtQuick

// Four bars like a phone's signal indicator; the spoken label lives on the row.
Row {
    id: bars
    property int level: 0
    readonly property int lit: level <= 0 ? 0 : Math.min(4, Math.ceil(level / 25))
    spacing: 2
    height: 18
    Accessible.ignored: true
    Repeater {
        model: 4
        delegate: Rectangle {
            required property int index
            width: 4
            height: 6 + index * 4
            anchors.bottom: parent.bottom
            radius: 1.5
            color: index < bars.lit ? Theme.text : Theme.selected
        }
    }
}
