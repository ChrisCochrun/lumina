import QtQuick 2.13

Rectangle {
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
}
