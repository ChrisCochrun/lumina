import QtQuick 2.13
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.13
import QtQuick.Layouts 1.2
import QtAudioEngine 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property string text: "GOOD"

    GridLayout {
        anchors.fill: parent
        columns: 3
        rowSpacing: 5
        columnSpacing: 0

        Controls.ToolBar {
            Layout.fillWidth: true
            Layout.columnSpan: 3
            id: toolbar
            RowLayout {
                anchors.fill: parent 

                Controls.ToolButton {
                    text: "Grid"
                }
                Controls.ToolButton {
                    text: "Solo"
                }
                Controls.ToolSeparator {}
                Item { Layout.fillWidth: true }
                Controls.ToolSeparator {}
                Controls.ToolButton {
                    text: "Effects"
                    icon.name: "image-auto-adjust"
                    onClicked: {}
                }
                Controls.ToolButton {
                    id: backgroundButton
                    text: "Background"
                    icon.name: "fileopen"
                    onClicked: backgroundType.open()
                }
            }
        }

        Item {
            /* Layout.preferredHeight: 200 */
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 3
        }

        Kirigami.Icon {
            source: "arrow-left"
            Layout.preferredWidth: 200
            Layout.preferredHeight: 200
            Layout.alignment: Qt.AlignRight
        }

        Presenter.Slide {
            Layout.preferredWidth: 900
            Layout.preferredHeight: width / 16 * 9
            Layout.alignment: Qt.AlignCenter
            textSize: width / 15
            text: root.text
        }

        Kirigami.Icon {
            source: "arrow-right"
            Layout.preferredWidth: 200
            Layout.preferredHeight: 200
            Layout.alignment: Qt.AlignLeft
        }

        Item {
            /* Layout.preferredHeight: 200 */
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 3
        }

    }
}
