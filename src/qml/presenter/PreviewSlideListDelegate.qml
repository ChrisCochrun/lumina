import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import QtWebEngine 1.10
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Item {
    id: root
    implicitHeight: Kirigami.Units.gridUnit * 6.5
    implicitWidth: Kirigami.Units.gridUnit * 9
    property bool showVidBG
    property var rootModel: model
    /* property var previewSlidesList: parent */
    /* Component.onCompleted: { */
    /*     if (model.videoBackground != "") */
    /*         SlideMod.thumbnailVideoRust(model.videoBackground, model.serviceItemId, index, SlideModel); */
    /* } */

    Rectangle {
        id: previewHighlight
        anchors.centerIn: parent
        width: parent.width
        height: parent.height - slidesTitle.height - 5
        border.color: Kirigami.Theme.highlightColor
        radius: 5
        color: {
            if (active || previewerMouse.containsMouse)
                Kirigami.Theme.highlightColor
            else
                Kirigami.Theme.backgroundColor
        }

        Presenter.PreviewSlide {
            id: previewSlideItem
            anchors.centerIn: parent
            implicitWidth: height / 9 * 16
            implicitHeight: parent.height - Kirigami.Units.smallSpacing * 2
            textSize: model.fontSize
            itemType: model.type
            imageSource: {
                if (model.videoBackground != "") {
                    return model.videoThumbnail;
                } else if (model.imageBackground.endsWith(".html")) {
                    return "";
                } else
                    return model.imageBackground;
            }
            chosenFont: model.font
            text: model.text
            pdfIndex: model.slideIndex

        }
        /* WebEngineView { */
        /*     id: web */
        /*     anchors.centerIn: parent */
        /*     implicitWidth: height / 9 * 16 */
        /*     implicitHeight: parent.height - Kirigami.Units.smallSpacing * 2 */
        /*     url: model.imageBackground.endsWith(".html") ? model.imageBackground : "" */
        /*     visible: model.imageBackground.endsWith(".html") */
        /* } */
    }

    Controls.Label {
        id: slidesTitle
        width: previewHighlight.width
        anchors.top: previewHighlight.bottom
        anchors.left: previewHighlight.left
        anchors.topMargin: Kirigami.Units.smallSpacing
        anchors.rightMargin: Kirigami.Units.smallSpacing * 2
        elide: Text.ElideRight
        text: ServiceItemC.getRust(serviceItemId, ServiceItemModel).name
        font.bold: true
    }

    Controls.Label {
        id: obsSceneLabel
        anchors.top: previewHighlight.top
        anchors.right: previewHighlight.right
        anchors.topMargin: Kirigami.Units.smallSpacing
        anchors.rightMargin: Kirigami.Units.smallSpacing
        /* elide: Text.ElideRight */
        background: Rectangle {
            visible: model.obsScene.length > 0
            color: Qt.lighter(Kirigami.Theme.backgroundColor, 1.2)
            radius: 3
        }

        horizontalAlignment: Text.AlignRight
        text: model.obsScene.length > 0 ? "OBS Scene: " + model.obsScene : ""
        font.pointSize: 8
        rightPadding: 4
        leftPadding: 4
        color: Kirigami.Theme.highlightColor
    }

    MouseArea {
        id: previewerMouse
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: {
            if (mouse.button === Qt.RightButton) {
                rightClickMenu.popup(mouse);
            } else {
                changeSlide(index);
                showPassiveNotification(model.serviceItemId);
            }
        }
        cursorShape: Qt.PointingHandCursor
        propagateComposedEvents: true

        /* Controls.ToolTip { */
        /*     text: model.obsScene.length > 0 ? "OBS Scene: " + model.obsScene : "" */
        /* } */

    }

    Controls.Menu {
        id: rightClickMenu

        Controls.Menu {
            id: obsMenu
            title: "Obs Scenes"
            enabled: ObsModel.connected
            Instantiator {
                model: ObsModel.scenes
                Kirigami.Action {
                    text: modelData
                    onTriggered: {
                        Utils.dbg("setting: " + modelData + " on index: " + rootModel.index);
                        Utils.dbg(rootModel.obsScene);
                        SlideModel.updateObsScene(rootModel.index, modelData);
                        ObsModel.setScene(modelData);
                    }
                }
                onObjectAdded: obsMenu.insertAction(index, object)
                onObjectRemoved: obsMenu.removeAction(object)
            }
        }
    }
}
