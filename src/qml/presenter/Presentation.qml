import QtQuick 2.15
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.13
import QtQuick.Layouts 1.2
/* import QtAudioEngine 1.15 */
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

FocusScope {
    id: root

    property var text
    property int textIndex: 0
    property string itemType: SlideObject.type
    property url imagebackground: SlideObject.imageBackground
    property url vidbackground: SlideObject.videoBackground

    property Item slide: previewSlide

    Item {
        id: keyHandler
        anchors.fill: parent
        focus: true
        Keys.onLeftPressed: previousSlideAction()
        Keys.onRightPressed: nextSlideAction()
        Keys.onUpPressed: previousSlideAction()
        Keys.onDownPressed: nextSlideAction()
        Keys.onSpacePressed: nextSlideAction()
    }

    /* Component.onCompleted: nextSlideAction() */

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
                    text: "Solo"
                    icon.name: "viewimage"
                    hoverEnabled: true
                }
                Controls.ToolButton {
                    text: "Grid"
                    icon.name: "view-app-grid-symbolic"
                    hoverEnabled: true
                }
                Controls.ToolButton {
                    text: "Details"
                    icon.name: "view-list-details"
                    hoverEnabled: true
                }
                Controls.ToolSeparator {}
                Item { Layout.fillWidth: true }
                Controls.ToolSeparator {}
                Controls.ToolButton {
                    text: "Repeat"
                    icon.name: "repeat"
                    onClicked: mainPage.loopVideo()
                }
                Controls.ToolButton {
                    text: "Effects"
                    icon.name: "image-auto-adjust"
                    hoverEnabled: true
                    onClicked: {}
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
            Layout.preferredWidth: 100
            Layout.preferredHeight: 200
            Layout.alignment: Qt.AlignRight
            MouseArea {
                anchors.fill: parent
                onPressed: previousSlideAction()
                cursorShape: Qt.PointingHandCursor
            }
        }

        Presenter.Slide {
            id: previewSlide
            Layout.preferredWidth: 700
            Layout.preferredHeight: width / 16 * 9
            Layout.minimumWidth: 300
            Layout.alignment: Qt.AlignCenter
            textSize: width / 15
            itemType: root.itemType
            imageSource: imagebackground
            videoSource: vidbackground
            audioSource: SlideObject.audio
            chosenFont: SlideObject.font
            text: SlideObject.text
            pdfIndex: SlideObject.pdfIndex
            preview: true 
        }

        Kirigami.Icon {
            source: "arrow-right"
            Layout.preferredWidth: 100
            Layout.preferredHeight: 200
            Layout.alignment: Qt.AlignLeft
            MouseArea {
                anchors.fill: parent
                onPressed: nextSlideAction()
                cursorShape: Qt.PointingHandCursor
            }
        }

        Item {
            Layout.fillWidth: true
        }

        RowLayout {
            spacing: 2
            Layout.preferredWidth: previewSlide.width - 50
            /* Layout.columnSpan: 3 */
            Kirigami.Icon {
                source: previewSlide.mpvIsPlaying ? "media-pause" : "media-play"
                Layout.preferredWidth: 25
                Layout.preferredHeight: 25
                visible: itemType === "video";
                MouseArea {
                    anchors.fill: parent
                    onPressed: SlideObject.playPause();
                    cursorShape: Qt.PointingHandCursor
                }
            }
            Controls.Slider {
                id: videoSlider
                visible: itemType === "video";
                Layout.fillWidth: true
                Layout.preferredHeight: 25
                from: 0
                to: previewSlide.mpvDuration
                value: previewSlide.mpvPosition
                live: true
                onMoved: changeVidPos(value);
            }

            Controls.Switch {
                text: "Loop"
                visible: itemType === "video";
                checked: previewSlide.mpvLoop === "inf" ? true : false
                onToggled: mainPage.loopVideo()
            }
        }

        Item {
            Layout.fillWidth: true
        }

        Item {
            /* Layout.preferredHeight: 200 */
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 3
        }

        ListView {
            id: previewSlidesList
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.columnSpan: 3
            orientation: ListView.Horizontal
            cacheBuffer: 900
            reuseItems: true

            model: serviceItemModel
            delegate: Rectangle {
                id: previewHighlight
                implicitWidth: 210
                implicitHeight: width / 16 * 9
                color: {
                    if (active || previewerMouse.containsMouse)
                        Kirigami.Theme.highlightColor
                    else
                        Kirigami.Theme.backgroundColor
                }

                Presenter.Slide {
                    id: previewSlideItem
                    anchors.centerIn: parent
                    implicitWidth: 200
                    implicitHeight: width / 16 * 9
                    textSize: width / 15
                    itemType: type
                    imageSource: backgroundType === "image" ? background : ""
                    videoSource: backgroundType === "video" ? background : ""
                    audioSource: ""
                    chosenFont: font
                    text: text
                    pdfIndex: 0
                    preview: true 
                    editMode: true 

                }

                Controls.Label {
                    id: slidesTitle
                    width: previewSlideItem.width
                    anchors.top: previewSlideItem.bottom
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.topMargin: 5
                    elide: Text.ElideRight
                    text: name
                    /* font.family: "Quicksand Bold" */
                }

                MouseArea {
                    id: previewerMouse
                    anchors.fill: parent
                    onClicked: changeServiceItem(index)
                    cursorShape: Qt.PointingHandCursor
                }


                Connections {
                    target: serviceItemModel
                    onDataChanged: if (active)
                        previewSlidesList.positionViewAtIndex(index, ListView.Center)
                }
            }
            Kirigami.WheelHandler {
                id: wheelHandler
                target: previewSlidesList
                filterMouseEvents: true
            }

        }
        Item {
            /* Layout.preferredHeight: 200 */
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 3
        }

    }

    Connections {
        target: SlideObject
        function onVideoBackgroundChanged() {
            if (SlideObject.videoBackground === "")
                stopVideo();
            else
                loadVideo();
        }
        function onIsPlayingChanged() {
            if(SlideObject.isPlaying)
                previewSlide.playVideo();
            pauseVideo();
        }
        /* function onAudioChanged() { */
        /*     showPassiveNotification("Audio should change"); */
        /*     previewSlide.playAudio(); */
        /* } */
    }

    Timer {
        interval: 500
        running: true
        repeat: true
        onTriggered: root.visible ? keyHandler.forceActiveFocus() : null
    }

    function pauseVideo() {
        previewSlide.pauseVideo();
    }

    function loadVideo() {
        /* showPassiveNotification("Loading Video " + vidbackground) */
        previewSlide.loadVideo();
    }

    function loopVideo() {
        previewSlide.loopVideo();
    }

    function stopVideo() {
        /* showPassiveNotification("Stopping Video") */
        previewSlide.stopVideo()
    }

    function nextSlideAction() {
        root.forceActiveFocus();
        if (currentServiceItem === totalServiceItems)
            return;
        const nextServiceItemIndex = currentServiceItem + 1;
        const nextItem = serviceItemModel.getItem(nextServiceItemIndex);
        print("currentServiceItem " + currentServiceItem);
        print("nextServiceItem " + nextServiceItemIndex);
        print(nextItem.name);
        const changed = SlideObject.next(nextItem);
        print(changed);
        if (changed) {
            currentServiceItem++;
            changeServiceItem(currentServiceItem);
            leftDock.changeItem();
        }
    }

    function nextSlide() {
        changeServiceItem(currentServiceItem++);
        print(slideItem);
    }

    function previousSlideAction() {
        root.forceActiveFocus();
        if (currentServiceItem === 0) {
            return;
        };
        const prevServiceItemIndex = currentServiceItem - 1;
        const prevItem = serviceItemModel.getItem(prevServiceItemIndex);
        print("currentServiceItem " + currentServiceItem);
        print("prevServiceItem " + prevServiceItemIndex);
        print(prevItem.name);
        const changed = SlideObject.previous(prevItem);
        print(changed);
        if (changed) {
            currentServiceItem--;
            changeServiceItem(currentServiceItem);
            leftDock.changeItem();
        }
    }

    function previousSlide() {
        changeServiceItem(--currentServiceItem);
        print(slideItem);
    }

    function changeSlide() {
        if (itemType === "song") {
            SlideObject.setText(root.text[textIndex]);
            print(root.text[textIndex]);
            textIndex++;
        } else if (itemType === "video") {
            clearText();
        }
        else if (itemType === "image") {
            clearText();
        }
    }

    function clearText() {
        SlideObject.setText("");
    }
}
