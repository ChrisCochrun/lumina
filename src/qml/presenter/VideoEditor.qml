import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import mpv 1.0
import org.presenter 1.0

Item {
    id: root

    property string type: "video"
    property var video
    property bool audioOn: true
    property bool editingRange: false

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

                Controls.Label {
                    text: "Title:"
                }
                Controls.TextField {
                    id: titleId
                    implicitWidth: 300
                    hoverEnabled: true
                    placeholderText: "Song Title..."
                    text: video.title
                    padding: 10
                    onEditingFinished: updateTitle(text);
                    background: Presenter.TextBackground {
                        control: titleId
                    }
                }

                Controls.CheckBox {
                    id: loopCheckBox
                    /* Layout.preferredWidth: 300 */
                    /* Layout.fillWidth: true */
                    Layout.rightMargin: 20

                    icon.name: "media-repeat-all"
                    text: "Repeat"
                    padding: 10
                    checked: video.loop
                    onToggled: updateLoop(!video.loop)
                    background: Presenter.TextBackground {
                        control: loopCheckBox
                    }
                }

                Controls.ToolSeparator {}

                Item { Layout.fillWidth: true }
                Controls.ToolSeparator {}
                /* Controls.ToolButton { */
                /*     text: "Effects" */
                /*     icon.name: "image-auto-adjust" */
                /*     hoverEnabled: true */
                /*     onClicked: {} */
                /* } */
                Controls.ToolButton {
                    id: fileButton
                    text: "File"
                    icon.name: "fileopen"
                    hoverEnabled: true
                    onClicked: fileType.open()
                }
            }
        }

        Item {
            Layout.columnSpan: 2
            Layout.fillWidth: true
            Layout.preferredHeight: width / 16 * 9
            Layout.alignment: Qt.AlignCenter
            Layout.topMargin: 10
            Layout.leftMargin: Kirigami.Units.largeSpacing
            Layout.rightMargin: Kirigami.Units.largeSpacing

            MpvObject {
                id: videoPreview
                width: parent.width
                height: parent.height
	        objectName: "mpv"
                useHwdec: true
                enableAudio: audioOn
                Component.onCompleted: mpvLoadingTimer.start()
                onPositionChanged: videoSlider.value = position
                onFileLoaded: {
                    /* showPassiveNotification(video.title + " has been loaded"); */
                    videoPreview.pause();
                }
            }

            RowLayout {
                anchors.top: videoPreview.bottom
                width: videoPreview.width
                height: videoTitleField.height
                spacing: 2
                Kirigami.Icon {
                    source: videoPreview.isPlaying ? "media-pause" : "media-play"
                    Layout.preferredWidth: 25
                    Layout.preferredHeight: 25
                    color: Kirigami.Theme.textColor
                    MouseArea {
                        anchors.fill: parent
                        onPressed: videoPreview.playPause()
                        cursorShape: Qt.PointingHandCursor
                    }
                }
                Controls.Slider {
                    id: videoSlider
                    Layout.fillWidth: true
                    Layout.preferredHeight: 25
                    from: 0
                    to: videoPreview.duration
                    /* value: videoPreview.postion */
                    live: true
                    onMoved: videoPreview.seek(value);
                }

                Controls.Label {
                    id: videoTime
                    text: new Date(videoPreview.position * 1000).toISOString().slice(11, 19);
                }
            }
        }

        Rectangle {
            id: videoRangeBox
            Layout.columnSpan: 2
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.alignment: Qt.AlignHCenter
            Layout.leftMargin: Kirigami.Units.largeSpacing
            Layout.rightMargin: Kirigami.Units.largeSpacing
            Layout.topMargin: Kirigami.Units.largeSpacing * 3
            Layout.bottomMargin: Kirigami.Units.largeSpacing * 3
            /* visible: editingRange */

            Kirigami.Theme.colorSet: Kirigami.Theme.Complementary

            color: Kirigami.Theme.backgroundColor
            border.width: 1
            border.color: Kirigami.Theme.disabledTextColor
            radius: 6

            ColumnLayout {
                anchors {
                    fill: parent
                    topMargin: 5
                    bottomMargin: 5
                    leftMargin: 5
                    rightMargin: 5
                }

                Controls.Label {
                    text: "Adjust start and end times:"
                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Presenter.RangedSlider {
                    id: videoLengthSlider
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: Kirigami.Units.smallSpacing * 3
                    /* Layout.leftMargin: 25 */
                    /* Layout.rightMargin: 25 */

                    from: 0
                    to: videoPreview.duration

                    firstInitialValue: video.startTime
                    secondInitialValue: video.endTime

                    onFirstMoved: videoPreview.seek(firstVisualPosition)
                    onSecondMoved: videoPreview.seek(secondVisualPosition)

                    onFirstReleased: updateStartTime(firstValue)
                    onSecondReleased: updateEndTime(secondValue)
                }

                RowLayout {
                    Layout.preferredWidth: parent.width
                    Layout.alignment: Qt.AlignLeft

                    ColumnLayout {
                        Layout.alignment: Qt.AlignLeft
                        Controls.Label {
                            text: "Start Time:"
                        }
                        Controls.TextField {
                            id: startTimeField
                            Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                            text: new Date(videoLengthSlider.firstVisualPosition * 1000).toISOString().slice(11, 19);
                            horizontalAlignment: TextInput.AlignHCenter
                            background: Presenter.TextBackground {
                                control: startTimeField
                            }
                        }
                    }

                    ColumnLayout {
                        Layout.alignment: Qt.AlignRight
                        Controls.Label {
                            text: "End Time:"
                        }
                        Controls.TextField {
                            id: endTimeField
                            Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                            text: new Date(videoLengthSlider.secondVisualPosition * 1000).toISOString().slice(11, 19);
                            horizontalAlignment: TextInput.AlignHCenter
                            background: Presenter.TextBackground {
                                control: endTimeField
                            }
                        }
                    }

                }
            }
        }

        Item {
            id: botEmpty
            Layout.fillHeight: true
        }
    }
    Timer {
        id: mpvLoadingTimer
        interval: 100
        onTriggered: {
            videoPreview.loadFile(video.filePath.toString());
        }
    }

    function changeVideo(index) {
        let vid = videoProxyModel.getVideo(index);
        root.video = vid;
        console.log(video.startTime);
        console.log(video.endTime);
        mpvLoadingTimer.restart();
        footerSecondText = video.filePath;
        footerFirstText = "File path: ";
    }

    function stop() {
        console.log("stopping video");
        videoPreview.pause();
        console.log("quit mpv");
    }

    function updateEndTime(value) {
        videoProxyModel.videoModel.updateEndTime(video.id, Math.min(value, videoPreview.duration));
        video.endTime = value;
    }

    function updateStartTime(value) {
        videoProxyModel.videoModel.updateStartTime(video.id, Math.max(value, 0));
        video.startTime = value;
    }

    function updateTitle(text) {
        changeTitle(text, false);
        videoProxyModel.videoModel.updateTitle(video.id, text);
        /* showPassiveNotification(video.title); */
    }

    function updateLoop(value) {
        /* changeStartTime(value, false); */
        let bool = videoProxyModel.videoModel.updateLoop(video.id, value);
        if (bool)
            video.loop = value;
        /* showPassiveNotification("Loop changed to: " + video.loop); */
    }

    function changeTitle(text, updateBox) {
        if (updateBox)
            videoTitleField.text = text;
        video.title = text;
    }
}
