import QtQuick 2.15
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
                count: innerModel.count
                newItemFunction: (function() {
                    songProxyModel.setFilterRegularExpression("");
                    innerModel.newSong();
                    libraryList.currentIndex = innerModel.count - 1;
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
                count: innerModel.count
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
                count: innerModel.count
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
                count: innerModel.count
                newItemFunction: (function() {
                    presProxyModel.setFilterRegularExpression("");
                })
                deleteItemFunction: (function(rows) {
                    presProxyModel.deletePresentations(rows)
                })

            }

            Presenter.LibraryItem {
                id: slideLibrary
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                Layout.preferredHeight: parent.height - 280
                /* proxyModel: presProxyModel */
                innerModel: slideModel
                libraryType: "slide"
                headerLabel: "Slides"
                itemIcon: "x-office-presentation-symbolic"
                /* itemSubtitle: model.path */
                count: innerModel.count
                newItemFunction: (function() {
                    if (!editMode)
                        editMode = true;
                    editSwitch(0, libraryType);
                })
            }

            ListModel {
                id: slideModel
                ListElement {
                    title: "test"
                    items: []
                }

                ListElement {
                    title: "Cool Slide"
                    items: []
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
        videoLibrary.libraryList.currentIndex = videoProxyModel.videoModel.count - 1;
        if (!editMode)
            editMode = true;
        editSwitch(videoLibrary.libraryList.currentIndex, "video");
    }

    function addImg(url) {
        imageProxyModel.newItem(url);
        selectedLibrary = "image";
        imageLibrary.libraryList.currentIndex = imageProxyModel.imageModel.count - 1;
        if (!editMode)
            editMode = true;
        editSwitch(imageLibrary.libraryList.currentIndex, "image");
    }

    function addPres(url) {
        console.log(pdf.status);
        console.log("FILE IS: " + url);
        let pageCount = 1;

        if (url.endsWith(".pdf")) {
            pdf.source = url;
            console.log(pdf.status);
            console.log("PAGECOUNT: " + pdf.pageCount);
            pageCount = pdf.pageCount;
        } else
            pageCount = 1;

        presProxyModel.presentationModel.newItem(url, pageCount);
        selectedLibrary = "presentation";
        presentationLibrary.libraryList.currentIndex = presProxyModel.presentationModel.count - 1;
        let presId = presentationLibrary.libraryList.currentIndex + 1;
        let pres = presProxyModel.presentationModel.getItem(presId);
        console.log(pres.id);
        if (!editMode)
            editMode = true;
        editSwitch(presId, "presentation");
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

    /* function addHtml(url) { */
    /*     console.log("adding an html"); */
    /*     var pageCount = 1; */
    /*     web.runJavaScript("Reveal.getSlides()", function(result) { */
    /*         let str = ''; */
    /*         for (const [p, val] of Object.entries(result[0])) { */
    /*             str += `${p}::${val}\n`; */
    /*         } */
    /*         console.log(str); */
    /*         pageCount = result.length; */
    /*         console.log(pageCount); */
    /*         presProxyModel.presentationModel.newItem(url, pageCount); */
    /*         selectedLibrary = "presentation"; */
    /*         presentationLibrary.libraryList.currentIndex = presProxyModel.presentationModel.count(); */
    /*         console.log(presProxyModel.getPresentation(presentationLibrary.libraryList.currentIndex)); */
    /*         const presentation = presProxyModel.getPresentation(presentationLibrary.libraryList.currentIndex); */
    /*         showPassiveNotification("newest image: " + presentation.title); */
    /*         if (!editMode) */
    /*             editMode = true; */
    /*         editSwitch("presentation", presentation); */
    /*     }); */
    /* } */
}
