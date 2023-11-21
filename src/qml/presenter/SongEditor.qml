import QtQuick 2.13
import QtQuick.Controls 2.15 as Controls
import Qt.labs.platform 1.1 as Labs
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Item {
    id: root

    property int songID
    property var song: songEditorModel

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
                    implicitWidth: root.width / 5 > 300 ? 300 : root.width / 5
                    editable: true
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateFont(currentText)
                    onAccepted: updateFont(currentText)
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

                Controls.SpinBox {
                    id: fontSizeBox
                    editable: true
                    from: 5
                    to: 150
                    height: parent.height
                    hoverEnabled: true
                    onValueModified: updateFontSize(value)
                    background: Presenter.TextBackground {
                        control: fontSizeBox
                    }
                }
                Controls.ComboBox {
                    id: hAlignmentBox
                    model: ["Left", "Center", "Right", "Justify"]
                    implicitWidth: 100
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateHorizontalTextAlignment(currentText.toLowerCase());
                    background: Presenter.TextBackground {
                        control: hAlignmentBox
                    }

                    indicator: Kirigami.Icon {
                        anchors {right: parent.right
                                 verticalCenter: parent.verticalCenter
                                 rightMargin: 2}
                        source: "arrow-down"
                        rotation: hAlignmentBox.down ? 180 : 0
                        color: hAlignmentBox.pressed ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor
                        
                        Behavior on rotation {
                            NumberAnimation {
                                easing.type: Easing.OutCubic
                                duration: 300
                            }
                        }
                    }


                    contentItem: Text {
                        leftPadding: 0
                        rightPadding: hAlignmentBox.indicator.width + hAlignmentBox.spacing

                        text: hAlignmentBox.displayText
                        font: hAlignmentBox.font
                        color: hAlignmentBox.pressed ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor;
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                    }
                }
                Controls.ComboBox {
                    id: vAlignmentBox
                    model: ["Top", "Center", "Bottom"]
                    implicitWidth: 100
                    hoverEnabled: true
                    /* flat: true */
                    onActivated: updateVerticalTextAlignment(currentText.toLowerCase());
                    background: Presenter.TextBackground {
                        control: vAlignmentBox
                    }

                    indicator: Kirigami.Icon {
                        anchors {right: parent.right
                                 verticalCenter: parent.verticalCenter
                                 rightMargin: 2}
                        source: "arrow-down"
                        rotation: vAlignmentBox.down ? 180 : 0
                        color: vAlignmentBox.pressed ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor
                        
                        Behavior on rotation {
                            NumberAnimation {
                                easing.type: Easing.OutCubic
                                duration: 300
                            }
                        }
                    }

                    contentItem: Text {
                        leftPadding: 0
                        rightPadding: vAlignmentBox.indicator.width + vAlignmentBox.spacing

                        text: vAlignmentBox.displayText
                        font: vAlignmentBox.font
                        color: vAlignmentBox.pressed ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor;
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                    }

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
                    id: backgroundButton
                    text: "Background"
                    icon.name: "fileopen"
                    hoverEnabled: true
                    onClicked: backgroundTypePopup.open()
                }
                Controls.ToolButton {
                    text: "Effects"
                    icon.name: "image-auto-adjust"
                    hoverEnabled: true
                    onClicked: {}
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
                            onClicked: updateBackground("video") & backgroundTypePopup.close()
                            hoverEnabled: true
                        }
                        Controls.ToolButton {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            text: "Image"
                            icon.name: "folder-pictures-symbolic"
                            onClicked: updateBackground("image") & backgroundTypePopup.close()
                            hoverEnabled: true
                        }
                    }
                }
            }
        }

        Controls.SplitView {
            id: songSplitView
            Layout.fillHeight: true
            Layout.fillWidth: true
            Layout.columnSpan: 2
            handle: Item {
                implicitWidth: 6
                Rectangle {
                    height: parent.height
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 1
                    color: parent.Controls.SplitHandle.hovered ? Qt.lighter(Kirigami.Theme.backgroundColor, 1.5) : Qt.darker(Kirigami.Theme.backgroundColor, 1.5)
                }
            }
            
            ColumnLayout {
                Controls.SplitView.fillHeight: true
                Controls.SplitView.preferredWidth: 400
                Controls.SplitView.minimumWidth: 300

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
                    background: Presenter.TextBackground {
                        control: songTitleField
                        errorCondition: song.title.length === 0
                    }
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
                    text: song.verseOrder
                    padding: 10
                    onEditingFinished: updateVerseOrder(text);
                    background: Presenter.TextBackground {
                        control: songVorderField
                        errorCondition: song.verseOrderError
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
                        background: Presenter.TextBackground {
                            control: lyricsEditor
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
                    background: Presenter.TextBackground {
                        control: songAuthorField
                    }
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
                        background: Presenter.TextBackground {
                            control: songAudioField
                            errorCondition: !song.audioExists
                        }
                        Controls.ToolTip {
                            text: song.audioExists ? "The audio that will be played for this song" : "The audio is missing or does not exists"
                        }
                    }

                    Controls.ToolButton {
                        id: audioPickerButton
                        Layout.fillHeight: true
                        text: "Audio"
                        icon.name: "folder-music-symbolic"
                        onClicked: updateAudioFile()
                        hoverEnabled: true
                        /* background: Presenter.TextBackground { */
                        /*     control: audioPickerButton */
                        /* } */
                    }
                }
            }

            ColumnLayout {
                Controls.SplitView.fillHeight: true
                Controls.SplitView.preferredWidth: 700
                Controls.SplitView.minimumWidth: 300

                Presenter.SongEditorSlideList {
                    id: songList
                    imageBackground: songEditorModel.backgroundType === "image" ? song.background : ""
                    videoBackground: songEditorModel.backgroundType === "video" ? song.background : ""
                    font: songEditorModel.font
                    fontSize: songEditorModel.fontSize
                    /* hTextAlignment: songEditorModel.horizontalTextAlignment */
                    /* vTextAlignment: songEditorModel.verticalTextAlignment */
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

    function newSong(index) {
        clearSlides();
        let thisSong = songProxyModel.getSong(index);
        songEditorModel.title = thisSong.title;
        songEditorModel.lyrics = thisSong.lyrics;
        songEditorModel.author = thisSong.author;
        songEditorModel.ccli = thisSong.ccli;
        songEditorModel.audio = thisSong.audio;
        songEditorModel.verseOrder = thisSong.vorder;
        songEditorModel.background = thisSong.background;
        songEditorModel.backgroundType = thisSong.backgroundType;
        songEditorModel.horizontalTextAlignment = thisSong.horizontalTextAlignment;
        songEditorModel.verticalTextAlignment = thisSong.verticalTextAlignment;
        songEditorModel.font = thisSong.font;
        songEditorModel.fontSize = thisSong.fontSize;
        songEditorModel.checkFiles();
        songID = thisSong.id;

        updateHorizontalTextAlignment("Center");
        changeSlideHAlignment("Center");
        updateVerticalTextAlignment("Center");
        changeSlideVAlignment("Center");
        updateFont("Noto Sans");
        changeSlideFont("Noto Sans", true);
        updateFontSize(50);
        changeSlideFontSize(50, true);
        updateLyrics("Lyrics");
        songList.loadVideo();
        console.log("New song with ID: " + song.id);
    }

    function changeSong(index) {
        console.log("Preparing to change song: " + index + 1 + " out of " + songProxyModel.songModel.count());
        if (songProxyModel.songModel.count() - 1 === index)
            newSong(index)
        else {
            clearSlides();
            const updatedSong = songProxyModel.getSong(index);
            console.log(updatedSong.vorder + " " + updatedSong.title + " " + updatedSong.audio);
            songEditorModel.title = updatedSong.title;
            songEditorModel.lyrics = updatedSong.lyrics;
            songEditorModel.author = updatedSong.author;
            songEditorModel.ccli = updatedSong.ccli;
            songEditorModel.audio = updatedSong.audio;
            songEditorModel.verseOrder = updatedSong.vorder;
            songEditorModel.background = updatedSong.background;
            songEditorModel.backgroundType = updatedSong.backgroundType;
            songEditorModel.horizontalTextAlignment = updatedSong.horizontalTextAlignment;
            songEditorModel.verticalTextAlignment = updatedSong.verticalTextAlignment;
            songEditorModel.font = updatedSong.font;
            songEditorModel.fontSize = updatedSong.fontSize;
            songEditorModel.checkVerseOrder();
            songEditorModel.checkFiles();
            songID = updatedSong.id;

            changeSlideHAlignment(song.horizontalTextAlignment);
            changeSlideVAlignment(song.verticalTextAlignment);
            changeSlideFont(song.font, true);
            changeSlideFontSize(song.fontSize, true)
            changeSlideText(songProxyModel.modelIndex(index).row);
            console.log("Changing to song: " + song.title + " with ID: " + songID);
            footerFirstText = "Song: ";
            footerSecondText = song.title;
            songList.loadVideo();
        }
    }

    function updateLyrics(lyrics) {
        songProxyModel.songModel.updateLyrics(songID, lyrics);
        /* songLyrics = lyrics; */
        clearSlides();
        changeSlideText(songID);
    }

    function updateTitle(title) {
        songProxyModel.songModel.updateTitle(songID, title)
        song.title = title;
    }

    function updateAuthor(author) {
        songProxyModel.songModel.updateAuthor(songID, author)
    }

    function updateAudio(audio) {
        songProxyModel.songModel.updateAudio(songID, audio)
    }

    function updateCcli(ccli) {
        songProxyModel.songModel.updateCcli(songID, ccli)
    }

    function updateVerseOrder(vorder) {
        songProxyModel.songModel.updateVerseOrder(songID, vorder)
        songEditorModel.verseOrder = vorder;
        songEditorModel.checkVerseOrder();
    }

    function updateAudioFile() {
        const file = fileHelper.loadFile("Pick Audio", "audio");
        songEditorModel.audio = file;
        songProxyModel.songModel.updateAudio(songID, file);
        songEditorModel.checkFiles();
    }

    function updateBackground(backgroundType) {
        songEditorModel.backgroundType = backgroundType;
        const file = fileHelper.loadFile("Pick Background", backgroundType);
        songEditorModel.background = file;
        songProxyModel.songModel.updateBackground(songID, file);
        songProxyModel.songModel.updateBackgroundType(songID, backgroundType);
        console.log("changed background");
    }


    function updateHorizontalTextAlignment(textAlignment) {
        changeSlideHAlignment(textAlignment);
        songProxyModel.songModel.updateHorizontalTextAlignment(songID, textAlignment);
    }

    function updateVerticalTextAlignment(textAlignment) {
        changeSlideVAlignment(textAlignment);
        songProxyModel.songModel.updateVerticalTextAlignment(songID, textAlignment)
    }

    function updateFont(font) {
        showPassiveNotification(font);
        changeSlideFont(font, false);
        songProxyModel.songModel.updateFont(songID, font);
        song.font = font;
    }

    function updateFontSize(fontSize) {
        changeSlideFontSize(fontSize, false);
        songProxyModel.songModel.updateFontSize(songID, fontSize);
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
        const verses = songProxyModel.getLyricList(id);
        verses.forEach(songList.appendVerse);
        /* songList.loadVideo(); */
    }

    function clearSlides() {
        songList.clear()
    }
}
