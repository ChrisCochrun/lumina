import QtQuick 2.13
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Labs
import QtQuick.Pdf 5.15
import QtQml.Models 2.15
import QtWebEngine 1.10
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Item {
    id: root

    property string selectedLibrary: "songs"
    property bool overlay: false
    property var videoexts: ["mp4", "webm", "mkv", "avi", "MP4", "WEBM", "MKV"]
    property var imgexts: ["jpg", "png", "gif", "jpeg", "JPG", "PNG", "webp", "gif"]
    property var presexts: ["pdf", "PDF", "odp", "pptx", "html"]

    property bool htmlLoaded: false

    Kirigami.Theme.colorSet: Kirigami.Theme.View

    Rectangle {
        anchors.fill: parent
        color: Kirigami.Theme.backgroundColor
        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            Presenter.LibraryItem {
                id: songLibrary
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                Layout.preferredHeight: parent.height - 280
                proxyModel: songProxyModel
                innerModel: songProxyModel.songModel
                libraryType: "song"
                headerLabel: "Songs"
                itemIcon: "folder-music-symbolic"
                /* itemSubtitle: model.author */
                count: innerModel.count()
                newItemFunction: (function() {
                    songProxyModel.setFilterRegularExpression("");
                    innerModel.newSong();
                    libraryList.currentIndex = innerModel.count() - 1;
                    if (!editMode)
                        editMode = true;
                    editSwitch(libraryList.currentIndex, "song");
                })
                deleteItemFunction: (function(rows) {
                    songProxyModel.deleteSongs(rows)
                })

                Component.onCompleted: selectedLibrary = "song";
            }

            Presenter.LibraryItem {
                id: videoLibrary
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                Layout.preferredHeight: parent.height - 280
                proxyModel: videoProxyModel
                innerModel: videoProxyModel.videoModel
                libraryType: "video"
                headerLabel: "Videos"
                itemIcon: "folder-videos-symbolic"
                /* itemSubtitle: model.path */
                count: innerModel.count()
                newItemFunction: (function() {
                    videoProxyModel.setFilterRegularExpression("");
                    newVideo.open();
                })
                deleteItemFunction: (function(rows) {
                    videoProxyModel.deleteVideos(rows)
                })

            }

            Presenter.NewVideo {
                id: newVideo
            }

            Timer {
                id: videoDLTimer
                interval: 3000
                running: !newVideo.sheetOpen
                onTriggered: {
                    newVideo.clear();
                }
            }

            Presenter.LibraryItem {
                id: imageLibrary
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                Layout.preferredHeight: parent.height - 280
                proxyModel: imageProxyModel
                innerModel: imageProxyModel.imageModel
                libraryType: "image"
                headerLabel: "Images"
                itemIcon: "folder-pictures-symbolic"
                /* itemSubtitle: model.path */
                count: innerModel.count()
                newItemFunction: (function() {
                    imageProxyModel.setFilterRegularExpression("");
                })
                deleteItemFunction: (function(rows) {
                    imageProxyModel.deleteImages(rows)
                })

            }

            Presenter.LibraryItem {
                id: presentationLibrary
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                Layout.preferredHeight: parent.height - 280
                proxyModel: presProxyModel
                innerModel: presProxyModel.presentationModel
                libraryType: "presentation"
                headerLabel: "Presentations"
                itemIcon: "x-office-presentation-symbolic"
                /* itemSubtitle: model.path */
                count: innerModel.count()
                newItemFunction: (function() {
                    presProxyModel.setFilterRegularExpression("");
                })
                deleteItemFunction: (function(rows) {
                    presProxyModel.deletePresentations(rows)
                })

            }

            Rectangle {
                id: slideLibraryPanel
                Layout.preferredHeight: 40
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop
                color: Kirigami.Theme.backgroundColor

                Controls.Label {
                    anchors.centerIn: parent
                    text: "Slides"
                }

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        if (selectedLibrary == "slides")
                            selectedLibrary = ""
                        else
                            selectedLibrary = "slides"
                        console.log(selectedLibrary)
                    }
                }
            }

            Rectangle {
                id: slideLibraryHeader
                z: 2
                Layout.preferredHeight: 40
                Layout.fillWidth: true
                /* width: parent.width */
                color: Kirigami.Theme.backgroundColor
                opacity: 1
                state: "deselected"

                states: [
                    State {
                        name: "deselected"
                        when: (selectedLibrary !== "slides")
                        PropertyChanges { target: slideLibraryHeader
                                          Layout.preferredHeight: 0
                                        }
                    },
                    State {
                        name: "selected"
                        when: (selectedLibrary == "slides")
                        PropertyChanges { target: slideLibraryHeader }
                    }
                ]

                transitions: Transition {
                    to: "*"
                    NumberAnimation {
                        target: slideLibraryList
                        properties: "preferredHeight"
                        easing.type: Easing.OutCubic
                        duration: 300
                    }
                }

                Kirigami.ActionToolBar {
                    height: parent.height
                    width: parent.width
                    display: Controls.Button.IconOnly
                    visible: selectedLibrary == "slides"
                    actions: [
                        Kirigami.Action {
                            icon.name: "document-new"
                            text: "New Slide"
                            tooltip: "Add a new slide"
                            onTriggered: slideLibraryList.newSlide()
                            /* visible: selectedLibrary == "slides" */
                        },
                        
                        Kirigami.Action {
                            displayComponent: Component {
                                Kirigami.SearchField {
                                    id: searchField
                                    height: parent.height
                                    width: parent.width - 40
                                    onAccepted: showPassiveNotification(searchField.text, 3000)
                                }
                            }
                            /* visible: selectedLibrary == "slides" */
                        }
                    ]

                    Behavior on height {
                        NumberAnimation {
                            easing.type: Easing.OutCubic
                            duration: 300
                        }
                    }
                }
            }

            ListView {
                id: slideLibraryList
                Layout.preferredHeight: parent.height - 240
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop
                state: "deselected"

                states: [
                    State {
                        name: "deselected"
                        when: (selectedLibrary !== "slides")
                        PropertyChanges { target: slideLibraryList
                                          Layout.preferredHeight: 0
                                        }
                    },
                    State {
                        name: "selected"
                        when: (selectedLibrary == "slides")
                        PropertyChanges { target: slideLibraryList }
                    }
                ]

                transitions: Transition {
                    to: "*"
                    NumberAnimation {
                        target: slideLibraryList
                        properties: "preferredHeight"
                        easing.type: Easing.OutCubic
                        duration: 300
                    }
                }

                Controls.ScrollBar.vertical: Controls.ScrollBar {
                    /* anchors.right: videoLibraryList.right */
                    /* anchors.leftMargin: 10 */
                    /* anchors.left: videoLibraryList.right */
                    active: hovered || pressed
                }

            }
        }

        DropArea {
            id: fileDropArea
            anchors.fill: parent
            onDropped: drop => {
                overlay = false;
                console.log("dropped");
                console.log(drop.urls);
                /* thumbnailer.loadFile(drop.urls[0]); */
                if (drop.urls.length > 1){
                    addFiles(drop.urls);
                } else if (drop.urls.length === 1)
                    addFile(drop.urls[0]);
                else if (drop.urls.length === 0)
                    console.log("stoppp it ya dum dum");
            }
            onEntered: {
                if (isDragFile(drag.urls[0]))
                    overlay = true;
            }
            onExited: overlay = false

        }

        Rectangle {
            id: fileDropOverlay
            color: overlay ? Kirigami.Theme.highlightColor : "#00000000"
            anchors.fill: parent
            border.width: 8
            border.color: overlay ? Kirigami.Theme.hoverColor : "#00000000"
        }

        // used for detecting number of pages without the need for PoDoFo
        PdfDocument {
            id: pdf
        }

        /* WebEngineView { */
        /*     id: web */
        /*     height: 0 */
        /*     width: 0 */
        /*     onLoadingChanged: { */
        /*         if (loadRequest.status == 2) */
        /*             addHtml(url); */
        /*     } */
        /* } */
    }

    function addVideo(url) {
        videoProxyModel.videoModel.newItem(url);
        selectedLibrary = "video";
        videoLibrary.libraryList.currentIndex = videoProxyModel.videoModel.count() - 1;
        if (!editMode)
            editMode = true;
        editSwitch(videoLibrary.libraryList.currentIndex, "video");
    }

    function addImg(url) {
        imageProxyModel.newItem(url);
        selectedLibrary = "image";
        imageLibrary.libraryList.currentIndex = imageProxyModel.imageModel.count() - 1;
        if (!editMode)
            editMode = true;
        editSwitch(imageLibrary.libraryList.currentIndex, "image");
    }

    function addPres(url) {
        console.log(pdf.status);
        let pageCount = 1;

        if (url.endsWith(".pdf")) {
            pdf.source = url;
            while (pdf.status != 2) {
                console.log(pdf.status);
                console.log("PAGECOUNT: " + pdf.pageCount);
                pageCount = pdf.pageCount;
            }
        } else
            pageCount = 1;

        presProxyModel.presentationModel.newItem(url, pageCount);
        selectedLibrary = "presentation";
        presentationLibrary.libraryList.currentIndex = presProxyModel.presentationModel.count() - 1;
        if (!editMode)
            editMode = true;
        editSwitch(presentationLibrary.libraryList.currentIndex, "presentation");
        pdf.source = "";
    }

    function isDragFile(item) {
        var extension = item.split('.').pop();
        var valid = false;

        if(extension) {
            console.log(extension);
            valid = true;
        }

        return valid;
    }

    function addFile(file) {
        let extension = file.split('.').pop();
        if (videoexts.includes(extension))
        {
            addVideo(file);
        }
        if (imgexts.includes(extension))
        {
            addImg(file);
        }
        if (presexts.includes(extension))
        {
            addPres(file);
        }
        
    }

    function addFiles(files) {
        showPassiveNotification("More than one file");
        for (let i = 0; i < files.length; i++) {
            let file = files[i];
            let ext = file.split('.').pop()
            if (videoexts.includes(ext))
            {
                addVideo(file);
            }
            if (imgexts.includes(ext))
            {
                console.log(file);
                addImg(file);
                console.log(file);
            }
            if (presexts.includes(ext))
            {
                addPres(file);
            }
        }
    }

    function addHtml(url) {
        console.log("adding an html");
        var pageCount = 1;
        web.runJavaScript("Reveal.getSlides()", function(result) {
            let str = '';
            for (const [p, val] of Object.entries(result[0])) {
                str += `${p}::${val}\n`;
            }
            console.log(str);
            pageCount = result.length;
            console.log(pageCount);
            presProxyModel.presentationModel.newItem(url, pageCount);
            selectedLibrary = "presentation";
            presentationLibrary.libraryList.currentIndex = presProxyModel.presentationModel.count();
            console.log(presProxyModel.getPresentation(presentationLibrary.libraryList.currentIndex));
            const presentation = presProxyModel.getPresentation(presentationLibrary.libraryList.currentIndex);
            showPassiveNotification("newest image: " + presentation.title);
            if (!editMode)
                editMode = true;
            editSwitch("presentation", presentation);
        });
    }
}
