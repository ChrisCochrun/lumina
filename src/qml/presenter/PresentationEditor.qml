import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import QtWebEngine 1.10
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property string type: "presentation"
    property var presentation
    property bool isHtml: presentation.filePath.endsWith(".html")

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
                    id: presentationTitleField
                    implicitWidth: 300
                    placeholderText: "Title..."
                    text: presentation.title
                    padding: 10
                    onEditingFinished: updateTitle(text);
                    background: Presenter.TextBackground {
                        control: fontBox
                    }
                }

                Controls.ComboBox {
                    model: ["PRESENTATIONS", "Center", "Right", "Justify"]
                    implicitWidth: 100
                    hoverEnabled: true
                    background: Presenter.TextBackground {
                        control: fontBox
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
                    text: "Effects"
                    icon.name: "presentation-auto-adjust"
                    hoverEnabled: true
                    onClicked: {}
                }
                Controls.ToolButton {
                    id: backgroundButton
                    text: "Select Presentation"
                    icon.name: "fileopen"
                    hoverEnabled: true
                    onClicked: backgroundType.open()
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
                            text: "Presentation"
                            icon.name: "emblem-presentations-symbolic"
                            onClicked: presentationFileDialog.open() & backgroundType.close()
                        }
                        Controls.ToolButton {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            text: "Presentation"
                            icon.name: "folder-pictures-symbolic"
                            onClicked: presentationFileDialog.open() & backgroundType.close()
                        }
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true
            /* Layout.minimumWidth: 300 */
            Layout.alignment: Qt.AlignCenter
            Layout.columnSpan: 2
            spacing: 5

            Item {
                id: topEmpty
                Layout.preferredHeight: 30
            }

            Image {
                id: presentationPreview
                Layout.preferredWidth: root.width - Kirigami.Units.largeSpacing
                Layout.preferredHeight: Layout.preferredWidth / 16 * 9
                Layout.alignment: Qt.AlignCenter
                fillMode: Image.PreserveAspectFit
                source: isHtml ? "" : presentation.filePath
                Component.onCompleted: {
                    updatePageCount(frameCount);
                    showPassiveNotification(presentation.pageCount);
                }
                visible: !isHtml
            }
            WebEngineView {
                id: webPresentationPreview
                Layout.preferredWidth: root.width - Kirigami.Units.largeSpacing
                Layout.preferredHeight: Layout.preferredWidth / 16 * 9
                Layout.alignment: Qt.AlignCenter
                url: isHtml ? presentation.filePath : ""
                visible: isHtml
                settings.playbackRequiresUserGesture: false
                backgroundColor: Kirigami.Theme.backgroundColor
            }
            RowLayout {
                Layout.fillWidth: true;
                Layout.alignment: Qt.AlignCenter
                Layout.leftMargin: 50
                Layout.rightMargin: 50
                Controls.ToolButton {
                    id: leftArrow
                    text: "Back"
                    icon.name: "back"
                    onClicked: {
                        if (isHtml) {
                            webPresentationPreview.runJavaScript("Reveal.prev()");
                        } else
                            presentationPreview.currentFrame = presentationPreview.currentFrame - 1
                    }
                }
                Item {
                    Layout.fillWidth: true
                }
                Controls.ToolButton {
                    id: rightArrow
                    text: "Next"
                    icon.name: "next"
                    onClicked: {
                        if (isHtml) {
                            webPresentationPreview.runJavaScript("Reveal.next()");
                        } else
                        presentationPreview.currentFrame = presentationPreview.currentFrame + 1
                    }
                }
            }
            Item {
                id: botEmpty
                Layout.fillHeight: true
            }

            Controls.TextArea {
                id: filePathLabel
                Layout.alignment: Qt.AlignBottom
                Layout.fillWidth: true
                text: presentation.filePath
                background: Item{}
                readOnly: true
                HoverHandler {
                    id: hoverHandler
                    enabled: false
                    cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.IBeamCursor
                }
            }

        }
    }

    function changePresentation(index) {
        let pres = presProxyModel.presentationModel.getItem(index);
        root.presentation = pres;
        console.log(pres.filePath.toString());
        updatePageCount(presentationPreview.frameCount);
        console.log("page count " + pres.pageCount);
        presentationPreview.currentFrame = 0;
    }

    function updateTitle(text) {
        changeTitle(text, false);
        presProxyModel.presentationModel.updateTitle(presentation.id, text);
        showPassiveNotification(presentation.title);
    }

    function changeTitle(text, updateBox) {
        if (updateBox)
            presentationTitleField.text = text;
        presentation.title = text;
    }

    function updatePageCount(pageCount) {
        let curPageCount = presentation.pageCount;
        if (curPageCount === presentation.pageCount)
            return;
        presentation.pageCount = pageCount;
        presProxyModel.presentationModel.updatePageCount(presentation.id, pageCount);
    }
}
