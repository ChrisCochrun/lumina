import QtQuick 2.15
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.15
import QtQuick.Layouts 1.15
import QtMultimedia 5.15
/* import QtAudioEngine 1.15 */
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property string imageBackground
    property string videoBackground
    property var hTextAlignment
    property var vTextAlignment
    property string font
    property real fontSize

    property bool playingVideo: false

    property ListModel songs: songModel
    property ListView songSlides: slideList

    property var firstItem

    Controls.ToolButton {
        id: playBackgroundButton
        anchors.top: parent.top
        anchors.left: parent.left
        text: playingVideo ? "Pause" : "Play"
        icon.name: playingVideo ? "media-pause" : "media-play"
        hoverEnabled: true
        onClicked: playPauseSlide();
    }


    Controls.ProgressBar {
        anchors.top: parent.top
        anchors.left: playBackgroundButton.right
        anchors.leftMargin: 10
        width: parent.width - playBackgroundButton.width - 10
        height: playBackgroundButton.height
        visible: firstItem.mpvDuration > 1;
        value: firstItem.mpvPosition
        to: firstItem.mpvDuration
    }

    ListView {
        id: slideList
        anchors.top: playBackgroundButton.bottom
        anchors.topMargin: 20
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.leftMargin: 10
        anchors.right: parent.right
        anchors.rightMargin: 20
        model: songModel
        clip: true
        cacheBuffer: 1900
        reuseItems: true
        spacing: Kirigami.Units.gridUnit
        synchronousDrag: true
        delegate: Loader {
            property var mpvDuration: representation.mpvDuration
            property var mpvPosition: representation.mpvPosition
            width: slideList.width
            height: width * 9 / 16
            Presenter.Slide {
                id: representation
                editMode: true
                imageSource: root.imageBackground
                videoSource: root.videoBackground
                hTextAlignment: root.hTextAlignment
                vTextAlignment: root.vTextAlignment
                chosenFont: font
                textSize: fontSize
                preview: true
                text: verse
                implicitWidth: slideList.width
                implicitHeight: width * 9 / 16
                itemType: "song"
            }
            function playPauseVideo() {
                representation.playPauseVideo();
            }
        }

        Kirigami.WheelHandler {
            id: wheelHandler
            target: slideList
            filterMouseEvents: true
        }

        Controls.ScrollBar.vertical: Controls.ScrollBar {
            parent: slideList.parent
            anchors.top: slideList.top
            anchors.left: slideList.right
            anchors.bottom: slideList.bottom
            active: hovered || pressed
        }


    }

    ListModel {
        id: songModel
    }

    function appendVerse(verse) {
        /* console.log("Let's append some verses") */
        /* console.log(verse); */
        /* showPassiveNotification(verse); */
        songModel.append({"verse": verse});
    }

    function updateHAlignment(alignment) {
        switch (alignment) {
        case "left" :
            root.hTextAlignment = Text.AlignLeft;
            break;
        case "center" :
            root.hTextAlignment = Text.AlignHCenter;
            break;
        case "right" :
            root.hTextAlignment = Text.AlignRight;
            break;
        case "justify" :
            root.hTextAlignment = Text.AlignJustify;
            break;
        }
    }

    function updateVAlignment(alignment) {
        switch (alignment) {
        case "top" :
            root.vTextAlignment = Text.AlignTop;
            break;
        case "center" :
            root.vTextAlignment = Text.AlignVCenter;
            break;
        case "bottom" :
            root.vTextAlignment = Text.AlignBottom;
            break;
        }
    }

    function playPauseSlide() {
        firstItem = slideList.itemAtIndex(0);
        console.log(firstItem);
        playingVideo = !playingVideo;
        /* firstItem.editMode = false; */
        firstItem.playPauseVideo();
    }

    function stopVideo() {
        representation.stopVideo();
    }

    function pauseVideo() {
        representation.pauseVideo();
    }

    function loadVideo() {
        showPassiveNotification("I'm loading the videos");
        for (var i = 0; i < slideList.count; ++i) {
            slideList.currentIndex = i;
            slideList.currentItem.representation.loadVideo();
            console.log(slideList.currentItem);
        }
        showPassiveNotification("I loaded the videos");
    }

    function clear() {
        songModel.clear()
    }
}
