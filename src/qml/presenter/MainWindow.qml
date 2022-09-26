import QtQuick 2.15
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.13
import QtQuick.Layouts 1.2
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Controls.Page {
    id: mainPage
    padding: 0

    // properties passed around for the slides
    property int currentServiceItem
    property url imageBackground: presentation.imageBackground
    property url videoBackground: presentation.vidBackground
    property string currentText: presentation.text
    property int blurRadius: 0
    property int totalServiceItems

    /* property var video */

    property int dragItemIndex
    property string dragItemTitle: ""
    property string dragItemType: ""
    property string dragItemText: ""
    property string dragItemAudio: ""
    property string dragItemBackgroundType: ""
    property string dragItemBackground: ""

    property bool editing: true

    property Item slideItem
    property var song
    property var draggedLibraryItem

    property bool songDragged: false

    property string editType

    Item {
        id: mainItem
        anchors.fill: parent

        Controls.SplitView {
            id: splitMainView
            anchors.fill: parent
            handle: Item{
                implicitWidth: 6
                Rectangle {
                    height: parent.height
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 1
                    color: Controls.SplitHandle.hovered ? Kirigami.Theme.hoverColor : Kirigami.Theme.backgroundColor
                }
            }

            Presenter.ServiceList {
                id: leftDock
                Controls.SplitView.preferredWidth: 200
                Controls.SplitView.maximumWidth: 300
            }
            
            Item {
                id: mainPageArea
                Controls.SplitView.fillWidth: true
                Controls.SplitView.minimumWidth: 100
                
                Presenter.Presentation { 
                    id: presentation
                    anchors.fill: parent
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
                Controls.SplitView.preferredWidth: libraryOpen ? 200 : 0
                Controls.SplitView.maximumWidth: 350
            }
 
        }
    }

    Presenter.PresentationWindow {
        id: pWindow
    }

    SongSqlModel {
        id: songsqlmodel
    }

    VideoSqlModel {
        id: videosqlmodel
    }

    ImageSqlModel {
        id: imagesqlmodel
    }

    PresentationSqlModel {
        id: pressqlmodel
    }

    ServiceItemModel {
        id: serviceItemModel
    }


    Item {
        id: keyHandler
        anchors.fill: parent
        focus: true
        Keys.onLeftPressed: presentation.previousSlideAction()
        Keys.onRightPressed: presentation.nextSlideAction()
        Keys.onUpPressed: presentation.previousSlideAction()
        Keys.onDownPressed: presentation.nextSlideAction()
        Keys.onSpacePressed: presentation.nextSlideAction()
    }

    function changeServiceItem(index) {
        const item = serviceItemModel.getItem(index);
        print("index grabbed: " + index);
        print(item);

        presentation.stopVideo();
        /* presentation.itemType = item.type; */
        print("Time to start changing");

        SlideObject.changeSlide(item);
        
        /* if (item.backgroundType === "video") */
        /* { */
        /*     presentation.loadVideo(); */
        /* } */

        presentation.textIndex = 0;
        /* presentation.changeSlide(); */

        print("Slide changed to: " + item.name);
    }

    function editSwitch(item) {
        if (editMode) {
            switch (editType) {
            case "song" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                imageEditor.visible = false;
                presentationEditor.visible = false;
                songEditor.visible = true;
                songEditor.changeSong(item);
                break;
            case "video" :
                presentation.visible = false;
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = false;
                videoEditor.visible = true;
                videoEditor.changeVideo(item);
                break;
            case "image" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                presentationEditor.visible = false;
                imageEditor.visible = true;
                imageEditor.changeImage(item);
                break;
            case "presentation" :
                presentation.visible = false;
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = true;
                presentationEditor.changePresentation(item);
                break;
            default:
                videoEditor.visible = false;
                videoEditor.stop();
                songEditor.visible = false;
                imageEditor.visible = false;
                presentationEditor.visible = false;
                presentation.visible = true;
                editMode = false;
            }
        } else {
            videoEditor.visible = false;
            videoEditor.stop();
            songEditor.visible = false;
            imageEditor.visible = false;
            presentationEditor.visible = false;
            presentation.visible = true;
            editMode = false;
            presenting = true;
        }
    }

    function present(present) {
        if (present)
        {
            presentation.loadVideo();
            print("For window: Screen is: " + pWindow.screen + " And selected screen is: " + presentationScreen);
            pWindow.showFullScreen();
            /* pWindow.screen = presentationScreen; */
            print("For window: Screen is: " + pWindow.screen + " And selected screen is: " + presentationScreen);
        }
        else
            pWindow.close();
    }

    function changeVidPos(pos) {
        presentation.slide.seek(pos);
        pWindow.slide.seek(pos);
    }
}
