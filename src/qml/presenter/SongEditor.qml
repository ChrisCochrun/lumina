import QtQuick 2.13
import QtQuick.Controls 2.15 as Controls
import Qt.labs.platform 1.1 as Labs
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property int songIndex
    property var song 

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

                Controls.ComboBox {
                    id: fontBox
                    model: Qt.fontFamilies()
                    implicitWidth: 300
                    editable: true
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateFont(currentText)
                }
                Controls.SpinBox {
                    id: fontSizeBox
                    editable: true
                    from: 5
                    to: 150
                    hoverEnabled: true
                    onValueModified: updateFontSize(value)
                }
                Controls.ComboBox {
                    id: hAlignmentBox
                    model: ["Left", "Center", "Right", "Justify"]
                    implicitWidth: 100
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateHorizontalTextAlignment(currentText.toLowerCase());
                }
                Controls.ComboBox {
                    id: vAlignmentBox
                    model: ["Top", "Center", "Bottom"]
                    implicitWidth: 100
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateVerticalTextAlignment(currentText.toLowerCase());
                }
                Controls.ToolButton {
                    text: "B"
                    hoverEnabled: true
                    visible: false
                }
                Controls.ToolButton {
                    text: "I"
                    hoverEnabled: true
                    visible: false
                }
                Controls.ToolButton {
                    text: "U"
                    hoverEnabled: true
                    visible: false
                }
                Controls.ToolSeparator {}
                Item { Layout.fillWidth: true }
                Controls.ToolSeparator {}
                Controls.ToolButton {
                    text: "Effects"
                    icon.name: "image-auto-adjust"
                    hoverEnabled: true
                    onClicked: {}
                }
                Controls.ToolButton {
                    id: backgroundButton
                    text: "Background"
                    icon.name: "fileopen"
                    hoverEnabled: true
                    onClicked: backgroundTypePopup.open()
                }

                Controls.Popup {
                    id: backgroundTypePopup
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
                            text: "Video"
                            icon.name: "emblem-videos-symbolic"
                            onClicked: videoFileDialog.open() & backgroundTypePopup.close()
                        }
                        Controls.ToolButton {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            text: "Image"
                            icon.name: "folder-pictures-symbolic"
                            onClicked: updateBackground("image") & backgroundTypePopup.close()
                        }
                    }
                }
            }
        }

        Controls.SplitView {
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 2
            handle: Item{
                implicitWidth: 6
                Rectangle {
                    height: parent.height
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 1
                    color: Controls.SplitHandle.hovered ? Kirigami.Theme.hoverColor : Kirigami.Theme.backgroundColor
                }
            }
            
            ColumnLayout {
                Controls.SplitView.fillHeight: true
                Controls.SplitView.preferredWidth: 500
                Controls.SplitView.minimumWidth: 500

                Controls.Label {
                    id: songTitleLabel
                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    leftPadding: 10
                    text: "Title"

                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Controls.TextField {
                    id: songTitleField

                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20

                    placeholderText: "Song Title..."
                    text: song.title
                    padding: 10
                    onEditingFinished: updateTitle(text);
                }

                Controls.Label {
                    id: songVorderLabel
                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    leftPadding: 10
                    text: "Verse Order <font color=\"Gray\"><i>format: V1 C1 V2 B1</i></font>"

                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Controls.TextField {
                    id: songVorderField

                    /* Layout.preferredWidth: 300 */
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20

                    placeholderText: "verse order..."
                    text: song.vorder
                    padding: 10
                    onEditingFinished: updateVerseOrder(text);
                    background: Rectangle {
                        color: songVorderField.enabled ? Kirigami.Theme.backgroundColor :
                            song.vorder.trim().length === 0 ?
                            Kirigami.Theme.negativeBackgroundColor :
                            Kirigami.Theme.backgroundColor
                        implicitWidth: parent.width
                        implicitHeight: parent.height
                        radius: 10
                        border.color: {
                            if (song.vorder.trim().length === 0)
                                return Kirigami.Theme.negativeTextColor
                            else if (songVorderField.enabled)
                                return Kirigami.Theme.highlightColor
                            else
                                return Kirigami.Theme.positiveColor
                        }
                    }
                }

                Controls.Label {
                    id: songLyricsLabel
                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    leftPadding: 10
                    text: "Lyrics"

                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Controls.ScrollView {
                    id: songLyricsField

                    Layout.preferredHeight: 2000
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.leftMargin: 20

                    rightPadding: 20

                    Controls.TextArea {
                        id: lyricsEditor
                        width: parent.width
                        placeholderText: "Put lyrics here..."
                        persistentSelection: true
                        text: song.lyrics
                        textFormat: TextEdit.PlainText
                        padding: 10
                        onEditingFinished: {
                            updateLyrics(text);
                            editorTimer.running = false;
                        }
                        onPressed: editorTimer.running = true
                        background: Rectangle {
                            color: Kirigami.Theme.backgroundColor
                            implicitWidth: parent.width
                            implicitHeight: parent.height
                            radius: 10
                            border.color: {
                                if (songVorderField.enabled)
                                    return Kirigami.Theme.highlightColor
                                else
                                    return Kirigami.Theme.positiveColor
                            }
                        }
                    }
                }

                Controls.Label {
                    id: songAuthorLabel
                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    leftPadding: 10
                    text: "Artist"

                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Controls.TextField {
                    id: songAuthorField

                    Layout.fillWidth: true
                    Layout.preferredWidth: 300
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20

                    placeholderText: "Author..."
                    text: song.author
                    padding: 10
                    onEditingFinished: updateAuthor(text)
                }

                Controls.Label {
                    id: songAudioLabel
                    Layout.preferredWidth: 300
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    leftPadding: 10
                    text: "Audio File"

                    Rectangle {
                        anchors.top: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        implicitHeight: Kirigami.Units.smallSpacing / 3
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 300
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20

                    Controls.TextField {
                        id: songAudioField
                        Layout.fillWidth: true
                        placeholderText: "Audio File..."
                        text: song.audio
                        padding: 10
                        onEditingFinished: showPassiveNotification(text)

                    }

                    Controls.ToolButton {
                        id: audioPickerButton
                        Layout.fillHeight: true
                        text: "Audio"
                        icon.name: "folder-music-symbolic"
                        onClicked: updateAudioFile()
                    }
                }
            }

            ColumnLayout {
                Controls.SplitView.fillHeight: true
                Controls.SplitView.preferredWidth: 700
                Controls.SplitView.minimumWidth: 300

                Presenter.SongEditorSlideList {
                    id: songList
                    imageBackground: song.backgroundType === "image" ? song.background : ""
                    videoBackground: song.backgroundType === "video" ? song.background : ""
                    Layout.preferredWidth: 500
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.bottomMargin: 20
                    Layout.topMargin: 10
                    Layout.rightMargin: 0
                    Layout.leftMargin: 10
                }
            }
        }
    }

    Timer {
        id: editorTimer
        interval: 1000
        repeat: true
        running: false
        onTriggered: {
            if (lyricsEditor.text === song.lyrics)
                return;
            updateLyrics(lyricsEditor.text);
        }
    }

    FileDialog {
        id: audioFileDialog
        title: "Please choose an audio file"
        folder: shortcuts.home
        selectMultiple: false
        nameFilters: ["Audio files (*.mp3 *.flac *.wav *.opus *.MP3 *.FLAC *.WAV *.OPUS)"]
        onAccepted: {
            updateAudioFile(audioFileDialog.fileUrls[0]);
            console.log("audio = " + audioFileDialog.fileUrls[0]);
        }
        onRejected: {
            console.log("Canceled")
        }

    }

    FileDialog {
        id: videoFileDialog
        title: "Please choose a background"
        folder: shortcuts.home
        selectMultiple: false
        nameFilters: ["Video files (*.mp4 *.mkv *.mov *.wmv *.avi *.MP4 *.MOV *.MKV)"]
        onAccepted: {
            updateBackground(videoFileDialog.fileUrls[0], "video");
            console.log("video background = " + videoFileDialog.fileUrls[0]);
        }
        onRejected: {
            console.log("Canceled")
        }

    }

    FileDialog {
        id: imageFileDialog
        title: "Please choose a background"
        folder: shortcuts.home
        selectMultiple: false
        nameFilters: ["Image files (*.jpg *.jpeg *.png *.JPG *.JPEG *.PNG)"]
        onAccepted: {
            updateBackground(imageFileDialog.fileUrls[0], "image");
            console.log("image background = " + imageFileDialog.fileUrls[0]);
        }
        onRejected: {
            console.log("Canceled")
        }

    }

    function newSong(index) {
        clearSlides();
        song = songProxyModel.getSong(index);

        changeSlideHAlignment("Center");
        changeSlideVAlignment("Center");
        changeSlideFont("Noto Sans", true);
        changeSlideFontSize(50, true)
        changeSlideText(songProxyModel.modelIndex(index).row);
        songList.loadVideo();
        console.log("New song with ID: " + song.id);
    }

    function changeSong(index) {
        clearSlides();
        song = songProxyModel.getSong(index);
        songIndex = song.id;

        changeSlideHAlignment(song.horizontalTextAlignment);
        changeSlideVAlignment(song.verticalTextAlignment);
        changeSlideFont(song.font, true);
        changeSlideFontSize(song.fontSize, true)
        changeSlideText(songProxyModel.modelIndex(index).row);
        songList.loadVideo();
        console.log("Changing to song: " + song.title + " with ID: " + song.id);
    }

    function updateLyrics(lyrics) {
        songProxyModel.songModel.updateLyrics(songIndex, lyrics);
        /* songLyrics = lyrics; */
        clearSlides();
        changeSlideText(songIndex);
    }

    function updateTitle(title) {
        songProxyModel.songModel.updateTitle(songIndex, title)
    }

    function updateAuthor(author) {
        songProxyModel.songModel.updateAuthor(songIndex, author)
    }

    function updateAudio(audio) {
        songProxyModel.songModel.updateAudio(songIndex, audio)
    }

    function updateCcli(ccli) {
        songProxyModel.songModel.updateCcli(songIndex, ccli)
    }

    function updateVerseOrder(vorder) {
        songProxyModel.songModel.updateVerseOrder(songIndex, vorder)
    }

    function updateAudioFile() {
        const file = fileHelper.loadFile("Pick Audio");
        songProxyModel.songModel.updateAudio(songIndex, file);
    }

    function updateBackground(backgroundType) {
        song.backgroundType = backgroundType;
        const file = fileHelper.loadFile("Pick Background");
        song.background = file;
        songProxyModel.songModel.updateBackground(songIndex, file);
        songProxyModel.songModel.updateBackgroundType(songIndex, backgroundType);
        console.log("changed background");
        /* if (backgroundType === "image") { */
        /*     //todo */
        /*     songList.videoBackground = ""; */
        /*     songList.imageBackground = background; */
        /* } else { */
        /*     //todo */
        /*     songList.imageBackground = ""; */
        /*     songList.videoBackground = background; */
        /*     songList.loadVideo(); */
        /* } */
    }


    function updateHorizontalTextAlignment(textAlignment) {
        changeSlideHAlignment(textAlignment);
        songProxyModel.songModel.updateHorizontalTextAlignment(songIndex, textAlignment);
    }

    function updateVerticalTextAlignment(textAlignment) {
        changeSlideVAlignment(textAlignment);
        songProxyModel.songModel.updateVerticalTextAlignment(songIndex, textAlignment)
    }

    function updateFont(font) {
        changeSlideFont(font, false);
        songProxyModel.songModel.updateFont(songIndex, font);
        song.font = font;
    }

    function updateFontSize(fontSize) {
        changeSlideFontSize(fontSize, false);
        songProxyModel.songModel.updateFontSize(songIndex, fontSize);
        song.fontSize = fontSize;
    }

    function changeSlideHAlignment(alignment) {
        switch (alignment) {
        case "left" :
            hAlignmentBox.currentIndex = 0;
            songList.hTextAlignment = Text.AlignLeft;
            break;
        case "center" :
            hAlignmentBox.currentIndex = 1;
            songList.hTextAlignment = Text.AlignHCenter;
            break;
        case "right" :
            hAlignmentBox.currentIndex = 2;
            songList.hTextAlignment = Text.AlignRight;
            break;
        case "justify" :
            hAlignmentBox.currentIndex = 3;
            songList.hTextAlignment = Text.AlignJustify;
            break;
        }
    }

    function changeSlideVAlignment(alignment) {
        switch (alignment) {
        case "top" :
            vAlignmentBox.currentIndex = 0;
            songList.vTextAlignment = Text.AlignTop;
            break;
        case "center" :
            vAlignmentBox.currentIndex = 1;
            songList.vTextAlignment = Text.AlignVCenter;
            break;
        case "bottom" :
            vAlignmentBox.currentIndex = 2;
            songList.vTextAlignment = Text.AlignBottom;
            break;
        }
    }

    function changeSlideFont(font, updateBox) {
        const fontIndex = fontBox.find(font);
        if (updateBox)
            fontBox.currentIndex = fontIndex;
        songList.font = font;
    }

    function changeSlideFontSize(fontSize, updateBox) {
        if (updateBox)
            fontSizeBox.value = fontSize;
        songList.fontSize = fontSize;
    }

    function changeSlideText(id) {
        /* console.log("Here are the verses: " + verses); */
        const verses = songProxyModel.getLyricList(id);
        verses.forEach(songList.appendVerse);
        /* songList.loadVideo(); */
    }

    function clearSlides() {
        songList.clear()
    }
}
