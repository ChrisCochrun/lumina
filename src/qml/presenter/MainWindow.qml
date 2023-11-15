import QtQuick 2.15
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.15
import QtQuick.Layouts 1.15
import QtWebEngine 1.10
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Controls.Page {
    id: mainPage
    padding: 0

    // properties passed around for the slides
    property int currentServiceItem
    property int currentSlide
    property int totalServiceItems: ServiceItemModel.rowCount()
    property int totalSlides: SlideMod.count()
    property url imageBackground: presentation.imageBackground
    property url videoBackground: presentation.vidBackground
    property url webSource
    property string currentText: presentation.text
    property int blurRadius: 0

    /* It's important to know that the INDEX is always the Index of the item
       in the Vector in Rust code. So, we pass the index from library lists
       around instead of here because getting the item requires us to use the
       correct Index from the QAbstractListModel to get a QModelIndex.*/
    property int dragItemIndex
    property string dragItemTitle: ""
    property string dragItemType: ""
    property string dragItemText: ""
    property string dragItemAudio: ""
    property string dragItemBackgroundType: ""
    property string dragItemBackground: ""
    property string dragItemFont: ""
    property string dragItemFontSize
    property int dragItemSlideNumber

    property bool editing: true

    property Item slideItem
    property var song
    property var draggedLibraryItem

    property var serviceItems: ServiceItemModel

    property bool songDragged: false

    property string editType

    property var currentWindow: presentation

    property var dragHighlightLine

    Component.onCompleted: {
        changeServiceItem(0);
        presentation.forceActiveFocus();
        /* const loaded = ServiceItemModel.loadLastSaved(); */
        /* if (!loaded) */
        /*     showPassiveNotification("Failed loading last file"); */
    }

    Item {
        id: mainItem
        anchors.fill: parent

        Controls.SplitView {
            id: splitMainView
            anchors.fill: parent
            handle: Item{
                implicitWidth: Kirigami.Units.gridUnit / 2
                Rectangle {
                    height: parent.height
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: parent.width / 4
                    color: parent.Controls.SplitHandle.hovered ? Kirigami.Theme.hoverColor : "#00000000"
                }
            }

            Presenter.ServiceList {
                id: leftDock
                Controls.SplitView.preferredWidth: Kirigami.Units.largeSpacing * 25
                Controls.SplitView.maximumWidth: Kirigami.Units.largeSpacing * 50
                z: 1
            }
            
            FocusScope {
                id: mainPageArea
                Controls.SplitView.fillWidth: true
                Controls.SplitView.fillHeight: true
                Controls.SplitView.minimumWidth: 100
                
                Presenter.Presentation { 
                    id: presentation
                    anchors.fill: parent
                    focus: true
                }

                Presenter.SongEditor {
                    id: songEditor
                    visible: false
                    anchors.fill: parent
                }

                Presenter.VideoEditor {
                    id: videoEditor
                    visible: false
                    anchors.fill: parent
                }

                Presenter.ImageEditor {
                    id: imageEditor
                    visible: false
                    anchors.fill: parent
                }

                Presenter.PresentationEditor {
                    id: presentationEditor
                    visible: false
                    anchors.fill: parent
                }
            }

            Presenter.Library {
                id: library
                Controls.SplitView.preferredWidth: libraryOpen ? Kirigami.Units.largeSpacing * 25 : 0
                Controls.SplitView.maximumWidth: Kirigami.Units.largeSpacing * 50
                visible: libraryOpen ? true : false
            }
            
        }
    }

    WebEngineView {
        id: web
        anchors.left: parent.right
        url: "file:///home/chris/org/lessons/2023_24_3_noah_lesson.html"
        visible: false
        WebEngineScript {
            name: "html2canvas"
            sourceUrl: "file:///home/chris/dev/lumina/src/qml/presenter/html2canvas.min.js"
        }
        onLoadingChanged: {
            if (loadRequest.status == 2)
                showPassiveNotification("yahoo?");
            getRevealThumbs("file:///home/chris/org/lessons/2023_24_3_noah_lesson.html");
        }
    }

    /* Loader { */
    /*     id: presWinLoader */
    /*     active: false */
    /*     sourceComponent: Presenter.PresentationWindow {} */
    /* } */
    /* Presenter.PresentationWindow { */
    /*     id: pWindow */
    /* } */

    SongProxyModel { id: songProxyModel }
    ImageProxyModel { id: imageProxyModel }
    PresentationProxyModel { id: presProxyModel }
    VideoProxyModel { id: videoProxyModel }
    ServiceThing { id: serviceThing } 
    FileHelper { id: fileHelper } 
    SlideHelper { id: slideHelper }
    SongEditor {
        id: songEditorModel
        /* songModel: songProxyModel.songModel() */
    }

    function changeServiceItem(index) {
        console.log("change-service-item: " + index);
        const item = ServiceItemC.getRust(index, ServiceItemModel);
        currentServiceItem = index;
        const slideId = SlideModel.findSlideIdFromServItm(index);
        currentSlide = slideId;
        const slide = SlideModel.getItemRust(slideId, SlideMod);
        console.log("index grabbed: " + index);
        console.log(slideId);
        console.log("Time to start changing");

        /* presentation.stopVideo(); */
        /* pWindow.stopVideo(); */
        /* presentation.itemType = item.type; */

        ServiceItemModel.activate(index);
        console.log("%%%%%%%%%");
        console.log(slide);
        /* SlideObject.changeSlide(slide, slideId); */
        slideHelper.chngSlide(slide, slideId, SlideObject);
        console.log("%%%%%%%%%");
        /* SlideObject.changeSlide(slide, slideId); */
        
        /* if (item.backgroundType === "video") */
        /* { */
        /*     presentation.loadVideo(); */
        /* } */

        presentation.textIndex = 0;
        /* ServiceItemModel.select(index); */
        /* presentation.changeSlide(); */

        console.log("Slide changed to: " + item.name);
    }

    function changeSlide(index) {
        console.log("index grabbed: " + index);
        const item = SlideModel.getItemRust(index, SlideMod);
        const isMoveDown = currentSlide < index;
        currentSlide = index;
        currentServiceItem = item.serviceItemId;
        console.log("index grabbed: " + index);
        console.log("html?: " + item.html);
        console.log("type: " + item.type);
        console.log("text: " + item.text);
        console.log("slide_index: " + item.slideIndex);
        console.log("slide_count: " + item.imageCount);
        if (item.html) {
            let index = item.slideIndex;
            let count = item.imageCount;
            if (index > 0 && index < count - 1) {
                console.log("I should advance revealy");
                if (isMoveDown)
                    presentation.revealNext()
                else
                    presentation.revealPrev()
                return
            }
        }

        /* presentation.stopVideo(); */
        /* pWindow.stopVideo(); */
        /* presentation.itemType = item.type; */
        console.log("Time to start changing");

        ServiceItemModel.activate(currentServiceItem);
        /* SlideObject.changeSlide(slide, slideId); */
        slideHelper.chngSlide(item, index, SlideObject);
        /* SlideMod.activate(index); */
        presentation.textIndex = 0;
        console.log("Slide changed to: ", item.imageBackground);
        activeServiceItem = ServiceItemC.getRust(currentServiceItem, ServiceItemModel).name;
    }

    function loopVideo() {
        presentation.loopVideo();
        pWindow.loopVideo();
    }

    function editSwitch(item, mode) {
        if (editMode) {
            switch (mode) {
            case "song" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                imageEditor.visible = false;
                presentationEditor.visible = false;
                songEditor.visible = true;
                songEditor.changeSong(item);
                currentWindow = songEditor;
                break;
            case "video" :
                presentation.visible = false;
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = false;
                videoEditor.visible = true;
                videoEditor.changeVideo(item);
                currentWindow = videoEditor;
                break;
            case "image" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                presentationEditor.visible = false;
                imageEditor.visible = true;
                imageEditor.changeImage(item);
                currentWindow = imageEditor;
                break;
            case "presentation" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = true;
                presentationEditor.changePresentation(item);
                currentWindow = presentationEditor;
                break;
            default:
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = false;
                presentation.visible = true;
                currentWindow = presentation;
                editMode = false;
                refocusPresentation();
                footerFirstText = presenting ? "Presenting..." : "Presentation Preview";
                footerSecondText = "";
            }
        } else {
            videoEditor.visible = false;
            videoEditor.stop();
            songEditor.visible = false;
            imageEditor.visible = false;
            presentationEditor.visible = false;
            presentation.visible = true;
            currentWindow = presentation;
            editMode = false;
            refocusPresentation();
            footerFirstText = presenting ? "Presenting..." : "Presentation Preview"
            footerSecondText = "";
        }
    }

    function present(present) {
        if (present)
        {
            PresWindow.showFullScreen();
            PresWindow.setSource("qrc:qml/presenter/PresentationWindow.qml")
            console.log(PresWindow);
            /* presWinLoader.active = true; */
        }
        else {
            PresWindow.close();

            /* presWinLoader.active = false; */
        }
    }

    function closeAll() { PresWindow.close() }

    function changeVidPos(pos) {
        presentation.slide.seek(pos);
        pWindow.slide.seek(pos);
    }

    function refocusPresentation() {
        presentation.forceActiveFocus();
        presentation.focusTimer = true;
    }

    function getRevealThumbs(file) {
        console.log(file);
        webSource = file;
        web.runJavaScript("
import('./html2canvas.min.js').then((html2canvas) => {
const screenshotTarget = document.body;

html2canvas(screenshotTarget).then((canvas) => {
    const base64image = canvas.toDataURL('image/png');
    return base64image;
});});", function(image) { console.log(image); });
            /*     web.runJavaScript(" */
            /*     const index */
            /*     for (let i = 0; i < index; i++) { */
            /*         Reveal.next(); */
            /*     }") */
    }
}
