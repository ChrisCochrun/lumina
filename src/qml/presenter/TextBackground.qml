import QtQuick 2.13
import QtGraphicalEffects 1.15

Rectangle {
    id: root
    // Used for 
    property var control
    property bool errorCondition

    color: Kirigami.Theme.backgroundColor
    implicitWidth: parent.width
    implicitHeight: parent.height
    radius: 10
    border.color: {
        if (control.enabled)
            return Kirigami.Theme.highlightColor
        else if (errorCondition)
            return Kirigami.Theme.negativeTextColor
        else
            return Kirigami.Theme.positiveColor
    }

    DropShadow {
        id: shadow
        source: root
        horizontalOffset: 2
        verticalOffset: 2
        radius: 3
        samples: 8
        color: "black"
    }
}
