import QtQuick 2.15
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.13
import QtQuick.Layouts 1.2
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
        visible: videoBackground.length() > 1;
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
        cacheBuffer: 900
        reuseItems: true
        spacing: Kirigami.Units.gridUnit
        /* flickDeceleration: 4000 */
        /* boundsMovement: Flickable.StopAtBounds */
        synchronousDrag: true
        delegate: Presenter.Slide {
            id: representation
            editMode: true
            imageSource: song.backgroundType = "image" ? song.background : ""
            videoSource: song.backgroundType = "video" ? song.background : ""
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
        print("Let's append some verses")
        print(verse);
        songModel.append({"verse": verse})
    }

    /* function loadVideo() { */
    /*     representation.loadVideo(); */
    /* } */

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
        print(firstItem);
        playingVideo = !playingVideo;
        /* firstItem.editMode = false; */
        firstItem.playPauseVideo();
    }

}
