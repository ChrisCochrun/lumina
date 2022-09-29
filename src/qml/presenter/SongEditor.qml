import QtQuick 2.13
import QtQuick.Controls 2.15 as Controls
import Qt.labs.platform 1.1 as Labs
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.2
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property int songIndex
    property var song 
    property string songLyrics

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
                    flat: true
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
                    flat: true
                    onActivated: updateHorizontalTextAlignment(currentText.toLowerCase());
                }
                Controls.ComboBox {
                    id: vAlignmentBox
                    model: ["Top", "Center", "Bottom"]
                    implicitWidth: 100
                    hoverEnabled: true
                    flat: true
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
                            onClicked: imageFileDialog.open() & backgroundTypePopup.close()
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
                        onClicked: audioFileDialog.open()
                    }
                }
            }

            ColumnLayout {
                Controls.SplitView.fillHeight: true
                Controls.SplitView.preferredWidth: 700
                Controls.SplitView.minimumWidth: 300

                Presenter.SlideEditor {
                    id: slideEditor
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
            if (lyricsEditor.text === songLyrics)
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
            print("audio = " + audioFileDialog.fileUrls[0]);
        }
        onRejected: {
            print("Canceled")
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
            print("video background = " + videoFileDialog.fileUrls[0]);
        }
        onRejected: {
            print("Canceled")
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
            print("image background = " + imageFileDialog.fileUrls[0]);
        }
        onRejected: {
            print("Canceled")
        }

    }

    function changeSong(index) {
        clearSlides();
        print(index);
        const s = songsqlmodel.getSong(index);
        song = s;
        songLyrics = s.lyrics;
        songIndex = index;

        if (song.backgroundType == "image") {
            slideEditor.videoBackground = "";
            slideEditor.imageBackground = song.background;
        } else {
            slideEditor.imageBackground = "";
            slideEditor.videoBackground = song.background;
            /* slideEditor.loadVideo(); */
        }

        changeSlideHAlignment(song.horizontalTextAlignment);
        changeSlideVAlignment(song.verticalTextAlignment);
        changeSlideFont(song.font, true);
        changeSlideFontSize(song.fontSize, true)
        changeSlideText(songIndex);
        print(s.title);
    }

    function updateLyrics(lyrics) {
        songsqlmodel.updateLyrics(songIndex, lyrics);
        songLyrics = lyrics;
        clearSlides();
        changeSlideText(songIndex);
    }

    function updateTitle(title) {
        songsqlmodel.updateTitle(songIndex, title)
    }

    function updateAuthor(author) {
        songsqlmodel.updateAuthor(songIndex, author)
    }

    function updateAudio(audio) {
        songsqlmodel.updateAudio(songIndex, audio)
    }

    function updateCcli(ccli) {
        songsqlmodel.updateCcli(songIndex, ccli)
    }

    function updateVerseOrder(vorder) {
        songsqlmodel.updateVerseOrder(songIndex, vorder)
    }

    function updateAudioFile(file) {
        songsqlmodel.updateAudio(songIndex, file);
    }

    function updateBackground(background, backgroundType) {
        song.backgroundType = backgroundType;
        song.background = background;
        songsqlmodel.updateBackground(songIndex, background);
        songsqlmodel.updateBackgroundType(songIndex, backgroundType);
        print("changed background");
        if (backgroundType === "image") {
            //todo
            slideEditor.videoBackground = "";
            slideEditor.imageBackground = background;
        } else {
            //todo
            slideEditor.imageBackground = "";
            slideEditor.videoBackground = background;
            slideEditor.loadVideo();
        }
    }


    function updateHorizontalTextAlignment(textAlignment) {
        changeSlideHAlignment(textAlignment);
        songsqlmodel.updateHorizontalTextAlignment(songIndex, textAlignment);
    }

    function updateVerticalTextAlignment(textAlignment) {
        changeSlideVAlignment(textAlignment);
        songsqlmodel.updateVerticalTextAlignment(songIndex, textAlignment)
    }

    function updateFont(font) {
        changeSlideFont(font, false);
        songsqlmodel.updateFont(songIndex, font);
        song.font = font;
    }

    function updateFontSize(fontSize) {
        changeSlideFontSize(fontSize, false);
        songsqlmodel.updateFontSize(songIndex, fontSize);
        song.fontSize = fontSize;
    }

    function changeSlideHAlignment(alignment) {
        switch (alignment) {
        case "left" :
            hAlignmentBox.currentIndex = 0;
            slideEditor.hTextAlignment = Text.AlignLeft;
            break;
        case "center" :
            hAlignmentBox.currentIndex = 1;
            slideEditor.hTextAlignment = Text.AlignHCenter;
            break;
        case "right" :
            hAlignmentBox.currentIndex = 2;
            slideEditor.hTextAlignment = Text.AlignRight;
            break;
        case "justify" :
            hAlignmentBox.currentIndex = 3;
            slideEditor.hTextAlignment = Text.AlignJustify;
            break;
        }
    }

    function changeSlideVAlignment(alignment) {
        switch (alignment) {
        case "top" :
            vAlignmentBox.currentIndex = 0;
            slideEditor.vTextAlignment = Text.AlignTop;
            break;
        case "center" :
            vAlignmentBox.currentIndex = 1;
            slideEditor.vTextAlignment = Text.AlignVCenter;
            break;
        case "bottom" :
            vAlignmentBox.currentIndex = 2;
            slideEditor.vTextAlignment = Text.AlignBottom;
            break;
        }
    }

    function changeSlideFont(font, updateBox) {
        const fontIndex = fontBox.find(font);
        if (updateBox)
            fontBox.currentIndex = fontIndex;
        slideEditor.font = font;
    }

    function changeSlideFontSize(fontSize, updateBox) {
        if (updateBox)
            fontSizeBox.value = fontSize;
        slideEditor.fontSize = fontSize;
    }

    function changeSlideText(id) {
        /* print("Here are the verses: " + verses); */
        const verses = songsqlmodel.getLyricList(id);
        verses.forEach(slideEditor.appendVerse);
        /* slideEditor.loadVideo(); */
    }

    function clearSlides() {
        slideEditor.songs.clear()
    }
}
