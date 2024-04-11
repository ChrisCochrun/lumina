import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Controls.BusyIndicator {
    id: root
    property color color
    visible: true

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
                model: 1

                Rectangle {
                    id: delegate
                    required property int index

                    x: item.width / 2 - width / 2
                    y: item.height / 2 - height / 2
                    implicitWidth: root.width
                    implicitHeight: root.width
                    radius: 500
                    color: root.color
                    opacity: delegate.scale
                    /* visible: root.visible */

                    /* Text { */
                    /*     text: delegate.scale + " & " + delegate.opacity */
                    /* } */

                    PropertyAnimation {
                        target: delegate
                        property: "scale"
                        running: root.visible && root.running
                        from: 0
                        to: 1
                        loops: Animation.Infinite
                        duration: 500 * index
                        easing.type: Easing.OutInExpo
                        easing.amplitude: 2.0
                    }


                    /* transform: [ */
                    /*     Translate { */
                    /*         y: -Math.min(item.width, item.height) * 0.5 */
                    /*         x: index */
                    /*     }, */
                    /*     Rotation { */
                    /*         angle: delegate.index / repeater.count * 360 */
                    /*         origin.x: 5 */
                    /*         origin.y: 5 */
                    /*     } */
                    /* ] */
                }
            }
        }
    }
}
