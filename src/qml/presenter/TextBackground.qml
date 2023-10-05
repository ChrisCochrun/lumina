import QtQuick 2.13
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami

Item {
    id: root
    property var control
    property bool errorCondition: false
    implicitWidth: control.width
    implicitHeight: control.height

    Rectangle {
        id: rect
        color: Kirigami.Theme.backgroundColor
        anchors.fill: parent
        radius: 10
        border.width: 0.5
        border.color: control.activeFocus ? Kirigami.Theme.highlightColor : (errorCondition ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.disabledTextColor)
    }

    DropShadow {
        id: shadow
        width: control.hovered || control.activeFocus ? parent.width : 0
        height: control.hovered || control.activeFocus ? parent.height : 0
        source: rect
        horizontalOffset: control.hovered || control.activeFocus ? 2 : 0
        verticalOffset: control.hovered || control.activeFocus ? 2 : 0
        radius: 3
        samples: 16
        color: "#AA000000"
    }
}
