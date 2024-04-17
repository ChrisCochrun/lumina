
import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import QtWebEngine 1.10
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Item {
    id: root

    property var creationObject

    GridLayout {
        id: mainLayout
        anchors.fill: parent
        columns: 2
        rowSpacing: 5
        columnSpacing: 0

        Controls.ToolBar {
            Layout.fillWidth: true
            Layout.columnSpan: 2
            id: toolbar

            RowLayout {
                anchors.fill: parent 

                Controls.TextField {
                    id: slideTitleField
                    implicitWidth: 300
                    placeholderText: "Title..."
                    text: "idk"
                    padding: 10
                    onEditingFinished: updateTitle(text);
                    background: Presenter.TextBackground {
                        control: fontBox
                    }
                }

                Controls.ComboBox {
                    id: alignBox
                    model: ["Left", "Center", "Right", "Justify"]
                    implicitWidth: 100
                    hoverEnabled: true
                    background: Presenter.TextBackground {
                        control: alignBox
                    }
                    indicator: Kirigami.Icon {
                        anchors {right: parent.right
                                 verticalCenter: parent.verticalCenter
                                 rightMargin: 2}
                        source: "arrow-down"
                        rotation: fontBox.down ? 180 : 0
                        color: fontBox.pressed ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor
                        
                        Behavior on rotation {
                            NumberAnimation {
                                easing.type: Easing.OutCubic
                                duration: 300
                            }
                        }
                    }
                }
                Controls.ToolSeparator {}
                Item { Layout.fillWidth: true }
                Controls.ToolSeparator {}
                Controls.ToolButton {
                    text: "Text Box"
                    icon.name: "insert-text-frame"
                    hoverEnabled: true
                    onClicked: creationObject = "text"
                }
                Controls.ToolButton {
                    id: backgroundButton
                    text: "Image"
                    icon.name: "insert-image"
                    hoverEnabled: true
                    onClicked: creationObject = "image"
                }

                Controls.Popup {
                    id: backgroundType
                    x: backgroundButton.x
                    y: backgroundButton.y + backgroundButton.height + 20
                    modal: true
                    focus: true
                    dim: false
                    background: Rectangle {
                        Kirigami.Theme.colorSet: Kirigami.Theme.Tooltip
                        color: Kirigami.Theme.backgroundColor
                        radius: 10
                        border.color: Kirigami.Theme.activeBackgroundColor
                        border.width: 2
                    }
                    closePolicy: Controls.Popup.CloseOnEscape | Controls.Popup.CloseOnPressOutsideParent
                    ColumnLayout {
                        anchors.fill: parent
                        Controls.ToolButton {
                            Layout.fillHeight: true
                            Layout.fillWidth: true
                            text: "Slide"
                            icon.name: "emblem-presentations-symbolic"
                            /* onClicked: slideFileDialog.open() & backgroundType.close() */
                        }
                        Controls.ToolButton {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            text: "Slide"
                            icon.name: "folder-pictures-symbolic"
                            /* onClicked: slideFileDialog.open() & backgroundType.close() */
                        }
                    }
                }
            }
        }

        Rectangle {
            id: slideCanvas
            Layout.fillHeight: true
            Layout.fillWidth: true
            /* Layout.minimumWidth: 300 */
            Layout.alignment: Qt.AlignCenter
            Layout.columnSpan: 2
            color: "white"

            MouseArea {
                id: canvasMouse
                anchors.fill: parent
            }
        }
    }

    function createObject(objectType) {
        let component = Qt.createComponent("TextBox.qml");
        if (component.status === Component.Ready || component.status === Component.Error) {
            finishCreation(component);
        } else {
            component.statusChanged.connect(finishCreation);
        }
    }

    function finishCreation(component) {
        if (component.status === Component.Ready) {
            var image = component.createObject(slideCanvas, {"x": 100, "y": 100});
            if (image === null) {
                console.log("Error creating image");
            }
        } else if (component.status === Component.Error) {
            console.log("Error loading component:", component.errorString());
        }
    }
}
