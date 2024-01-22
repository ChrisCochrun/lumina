import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0
import Qt.labs.settings 1.0

Kirigami.OverlaySheet {
    id: root

    property bool ytdlLoaded: false

    header: Kirigami.Heading {
        text: "Add a local video or download a new one"
    }

    ColumnLayout {
        Controls.ToolBar {
            id: toolbar
            Layout.fillWidth: true
            RowLayout {
                anchors.fill: parent
                Controls.Label {
                    id: videoInputLabel
                    text: "Video"
                }

                Controls.TextField {
                    id: videoInput
                    Layout.fillWidth: true
                    hoverEnabled: true
                    placeholderText: "Download a video or enter one..."
                    text: ""
                    onEditingFinished: videoInput.text.startsWith("http") ? loadVideo() : showPassiveNotification("Coach called, this isn't it.");
                    background: Presenter.TextBackground {}
                }

                Controls.ToolButton {
                    id: localButton
                    text: videoInput.text.startsWith("http") ? "Download" : "Local Video"
                    icon.name: "folder-videos-symbolic"
                    hoverEnabled: true
                    onClicked: videoInput.text.startsWith("http") ? loadVideo() : videoFileDialog.open()
                }
            }
        }

        Item {
            id: centerItem
            Layout.preferredHeight: ytdl.loaded ? Kirigami.Units.gridUnit * 20 : Kirigami.Units.gridUnit * 5
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignHCenter
            visible: true

            Controls.BusyIndicator {
                id: loadingIndicator
                anchors.centerIn: parent
                running: ytdl.loading
            }

            Ytdl {
                id: ytdl
                loaded: false
                loading: false
            }

            /* Rectangle { */
            /*     color: "blue" */
            /*     anchors.fill: parent */
            /* } */
            ColumnLayout {
                id: loadedItem
                anchors.fill: parent
                visible: ytdl.loaded
                Image {
                    id: thumbnailImage
                    Layout.alignment: Qt.AlignLeft | Qt.AlignTop
                    Layout.fillWidth: true
                    Layout.preferredHeight: width * 9 / 16
                    source: ytdl.thumbnail
                    fillMode: Image.PreserveAspectFit
                    clip: true

                    Item {
                        id: mask
                        anchors.fill: thumbnailImage
                        visible: false

                        Rectangle {
                            color: "white"
                            radius: 20
                            anchors.centerIn: parent
                            width: thumbnailImage.paintedWidth
                            height: thumbnailImage.paintedHeight
                        }
                    }
                    OpacityMask {
                        anchors.fill: thumbnailImage
                        source: thumbnailImage
                        maskSource: mask
                    }

                }
                Item {
                    Layout.alignment: Qt.AlignTop | Qt.AlignLeft
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Controls.Label {
                        id: videoTitle
                        text: ytdl.title
                        wrapMode: Text.WordWrap
                        elide: Text.ElideRight
                    }
                }

                Item {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Kirigami.Units.gridUnit * 2
                    Controls.ToolButton {
                        anchors.right: parent.right
                        text: "Ok"
                        icon.name: "check-filled"
                        hoverEnabled: true
                        onClicked: {
                            videoProxyModel.videoModel.newItem(ytdl.file);
                            clear();
                            root.close();
                        }
                    }
                }
            }

        }

        FileDialog {
            id: videoFileDialog
            title: "Please choose a video"
            folder: shortcuts.home
            selectMultiple: false
            nameFilters: ["Video files (*.mp4 *.mkv *.mov *.wmv *.avi *.MP4 *.MOV *.MKV)"]
            onAccepted: {
                console.log("video = " + videoFileDialog.fileUrls[0]);
                addVideo(videoFileDialog.fileUrls[0]);
                root.close();
            }
            onRejected: {
                console.log("Canceled")
            }

        }

        Timer {
            id: videoDLTimer
            interval: 3000
            running: !root.sheetOpen
            onTriggered: {
                clear();
            }
        }

    }

    function loadVideo() {
        ytdl.getVideo(videoInput.text)
        /* if (ytdl.getVideo(videoInput.text)) */
        /*     loadingIndicator.visible = true; */
    }

    function clear() {
        videoInput.text = "";
        ytdl.title = "";
        ytdl.thumbnail = "";
        ytdl.loaded = false;
        ytdl.loading = false;
    }
}
