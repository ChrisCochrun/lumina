import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import QtMultimedia 5.15
/* import QtAudioEngine 1.15 */
import QtWebEngine 1.10
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0
import mpv 1.0

Item {
    id: root

    // Let's make this slide editable
    property bool editMode: false

    // These properties are for the slides visuals
    property real textSize: 50
    property bool dropShadow: false
    property url imageSource
    property url webSource
    property bool htmlVisible: false
    property url videoSource
    property url audioSource
    property bool vidLoop
    property real vidStartTime
    property real vidEndTime
    property int pdfIndex
    property string chosenFont: "Quicksand"
    property string text
    property string audioError
    property color backgroundColor
    property var hTextAlignment: Text.AlignHCenter
    property var vTextAlignment: Text.AlignVCenter

    //these properties are for giving video info to parents
    property int mpvPosition: mpv.position
    property int mpvDuration: mpv.duration
    property var mpvLoop: mpv.loop
    property bool mpvIsPlaying: mpv.isPlaying

    // These properties help to determine the state of the slide
    property string itemType
    property bool preview: false

    implicitWidth: 1920
    implicitHeight: 1080

    Rectangle {
        id: basePrColor
        anchors.fill: parent
        color: "black"

        MpvObject {
            id: mpv
	    /* objectName: "mpv" */
            anchors.fill: parent
            useHwdec: true
            enableAudio: !preview
            Component.onCompleted: mpvLoadingTimer.start()
            onFileLoaded: {
                /* showPassiveNotification(videoSource + " has been loaded"); */
                if (itemType == "song")
                    mpv.setProperty("loop", "inf");
                else if (vidLoop)
                    mpv.setProperty("loop", "inf");
                else
                    mpv.setProperty("loop", "no");
                /* showPassiveNotification(mpv.getProperty("loop")); */
                mpv.setProperty("sid", "no");

            }
            /* onIsPlayingChanged: showPassiveNotification(mpv.getProperty("pause")) */

            MouseArea {
                id: playArea
                anchors.fill: parent
                enabled: editMode
                /* onPressed: mpv.playPause(); */
                cursorShape: preview ? Qt.ArrowCursor : Qt.BlankCursor
            }
        }

        Timer {
            id: mpvLoadingTimer
            interval: 100
            onTriggered: {
                /* showPassiveNotification("YIPPEEE!") */
                if (vidStartTime > 0 && vidStartTime < vidEndTime) {
                    /* mpv.seek(vidStartTime); */
                    let start = "+" + vidStartTime;
                    let end = "+" + vidEndTime;
                    mpv.setProperty("start", start);
                    mpv.setProperty("end", end);
                    Utils.dbg("Setting position to: " + mpv.position);
                } else {
                    mpv.setProperty("start", "none");
                    mpv.setProperty("end", "none");
                }
                mpv.loadFile(videoSource.toString());
                Utils.dbg("Setting position to: " + vidStartTime + " and end is " + vidEndTime);
                if (editMode) {
                    console.log("WHY AREN'T YOU PASUING!");
                    pauseTimer.restart();
                }
                blackTimer.restart();
            }
        }

        Timer {
            id: pauseTimer
            interval: 300
            onTriggered: mpv.pause()
        }

        Timer {
            id: blackTimer
            interval: 400
            onTriggered: {
                black.visible = false;
            }
        }

        Rectangle {
            id: black
            color: "Black"
            anchors.fill: parent
            visible: false
        }

        MpvObject {
            id: audio
            useHwdec: true
            enableAudio: true
            // embeded mpv allows to set commandline propertys using the options/<name>
            // syntax. This could be abstracted later, but for now this works.
            Component.onCompleted: audio.setProperty("options/audio-display", "no");
        }

        Image {
            id: backgroundImage
            anchors.fill: parent
            source: imageSource
            fillMode: itemType == "song" ? Image.PreserveAspectCrop : Image.PreserveAspectFit
            clip: true
            visible: webSource.length == 0
            currentFrame: pdfIndex
        }

        FastBlur {
            id: imageBlue
            anchors.fill: parent
            source: imageSource === "" ? mpv : backgroundImage
            radius: blurRadius

            /* Controls.Label { */
            /*     text: Math.max(root.width, 1000) / 1000 * Math.max(root.textSize, 50) */
            /*     horizontalAlignment: hTextAlignment */
            /*     verticalAlignment: vTextAlignment */
            /*     anchors.top: parent.top */
            /*     anchors.left: parent.left */
            /*     anchors.topMargin: 10 */
            /*     anchors.bottomMargin: 10 */
            /*     anchors.leftMargin: 10 */
            /*     anchors.rightMargin: 10 */
            /*     visible: RSettings.debug */
            /* } */

            Controls.Label {
                id: lyrics
                text: root.text
                /* text: root.textSize */
                /* text: root.width / 1000 * root.textSize */
                font.pixelSize: Math.max(root.width, 1000) / 1000 * Math.max(root.textSize, 50)
                /* minimumPointSize: 5 */
                fontSizeMode: Text.Fit
                font.family: chosenFont
                horizontalAlignment: hTextAlignment
                verticalAlignment: vTextAlignment
                style: Text.Raised
                wrapMode: Text.WordWrap
                anchors.fill: parent
                anchors.topMargin: 10
                anchors.bottomMargin: 10
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                clip: true

                layer.enabled: true
                layer.effect: DropShadow {
                    horizontalOffset: 5
                    verticalOffset: 5
                    radius: 11.0
                    samples: 24
                    color: "#80000000"
                }
            }
        }

        WebEngineView {
            id: web
            anchors.fill: parent
            url: webSource
            visible: htmlVisible
            enabled: htmlVisible
            zoomFactor: preview ? 0.25 : 1.0
            onLoadingChanged: {
                if (loadRequest.status == 2)
                    showPassiveNotification("yahoo?");
            }
            settings.playbackRequiresUserGesture: false
            audioMuted: root.preview

            /* function moveToSlideIndex(index) { */
            /*     web.runJavaScript(" */
            /*     const index */
            /*     for (let i = 0; i < index; i++) { */
            /*         Reveal.next(); */
            /*     }") */
            /* } */
        }
    }

    Connections {
        target: SlideObject
        function onRevealNext() {
            console.log("revealNext")
            web.runJavaScript("Reveal.next()")
        }
        function onRevealPrev() {
            console.log("revealPrev")
            web.runJavaScript("Reveal.prev()")
        }
    }

    function changeText(text) {
        lyrics.text = text
    }

    function loopVideo() {
        if (mpv.getProperty("loop") === "inf") {
            showPassiveNotification("already looping");
            mpv.setProperty("loop", "no");
        }
        else {
            mpv.setProperty("loop", "inf");
            showPassiveNotification("looping video");
        }
    }

    function loadVideo() {
        mpvLoadingTimer.restart()
    }

    function playAudio() {
        audio.loadFile(audioSource.toString());
        showPassiveNotification("Audio should change");
    }

    function stopAudio() {
        audio.stop();
    }

    function stopVideo() {
        mpv.stop();
        black.visible = true;
        console.log("Stopped video");
    }

    function seek(pos) {
        mpv.seek(pos);
    }

    function quitMpv() {
        mpv.quit();
    }

    function pauseVideo() {
        mpv.pause();
    }

    function playPauseVideo() {
        mpv.playPause();
    }

    function playVideo() {
        mpv.play();
    }

    function revealNext() {
        web.runJavaScript("Reveal.next()")
    }

    function revealPrev() {
        web.runJavaScript("Reveal.prev()")
    }
}
