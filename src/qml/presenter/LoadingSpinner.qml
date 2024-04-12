import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Controls.BusyIndicator {
    id: root
    property color color
    visible: root.running

    contentItem: Item {
        implicitWidth: 64
        implicitHeight: 64

        Item {
            id: item
            x: parent.width / 2 - 32
            y: parent.height / 2 - 32
            width: 64
            height: 64
            opacity: root.running ? 1 : 0

            Behavior on opacity {
                OpacityAnimator {
                    duration: 250
                }
            }

            Repeater {
                id: repeater
                model: 4

                Rectangle {
                    id: delegate
                    required property int index

                    x: item.width / 2 - width / 2
                    y: item.height / 2 - height / 2
                    height: width
                    radius: 500
                    color: root.color

                    PropertyAnimation {
                        target: delegate
                        property: "width"
                        running: root.running
                        from: 0 - (index * 100)
                        to: root.width * 1.8
                        loops: Animation.Infinite
                        duration: 1400
                        easing.type: Easing.InSine
                    }


                    PropertyAnimation {
                        target: delegate
                        property: "opacity"
                        running: root.running
                        from: 1
                        to: 0
                        loops: Animation.Infinite
                        duration: 1400
                        easing.type: Easing.InSine
                    }
                }
            }
        }
    }
}
