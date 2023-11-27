import QtQuick 2.13
/* import QtQuick.Dialogs 1.0 */
import QtQuick.Controls 2.12 as Controls
/* import QtQuick.Window 2.15 */
import QtQuick.Layouts 1.15
import QtQuick.Shapes 1.15
import QtQml.Models 2.15
/* import QtQml.Models 2.12 */
/* import QtMultimedia 5.15 */
/* import QtAudioEngine 1.15 */
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Item {
    id: root
    property var selectedItem: serviceItemList.selected
    property var hlItem 

    Rectangle {
        id: bg
        color: Kirigami.Theme.backgroundColor
        anchors.fill: parent
    }

    FastBlur {
        id: backgroundBlur
        source: ShaderEffectSource {
            sourceItem: bg
            sourceRect: Qt.rect(0, 0, backgroundBlur.width, backgroundBlur.height)
        }
        anchors.fill: parent
        radius: 82
        opacity: 0.60
    }

    ColumnLayout {
        id: layout
        anchors.fill: parent

        Rectangle {
            id: headerBackground
            color: Kirigami.Theme.backgroundColor
            height: 40
            opacity: 1.0
            Layout.fillWidth: true

            Kirigami.Heading {
                id: serviceTitle
                text: "Service List"
                anchors.centerIn: headerBackground 
                padding: 5
                level: 3
            }
        }

        DropArea {
            id: serviceDropEnd
            Layout.fillHeight: true
            Layout.fillWidth: true
            onDropped: (drag) => {
                console.log("DROPPED AT END");
                loadingItem.visible = true;
                appendItem(dragItemType,
                           dragItemIndex);
                dropHighlightLine.visible = false;
                loadingItem.visible = false;
            }

            keys: ["library"]

            onEntered: (drag) => {
                if (drag.keys[0] === "library") {
                    dropHighlightLine.visible = true;
                    var lastItem = serviceItemList.itemAtIndex(ServiceItemModel.count - 1);
                    dropHighlightLine.y = lastItem.y + lastItem.height;
                }
            }

            Component {
                id: delegate
                Kirigami.AbstractListItem {
                    id: serviceListItem
                    implicitWidth: serviceItemList.width
                    height: Kirigami.Units.gridUnit * 2

                    property var selectedItems

                    DropArea {
                        id: serviceDrop
                        anchors.fill: parent

                        onEntered: (drag) => {
                            if (drag.keys[0] === "library") {
                                dropHighlightLine.visible = true;
                                dropHighlightLine.y = serviceDrop.mapToItem(
                                    serviceItemList,0,0).y - 2;
                            }
                        }

                        onDropped: (drag) => {
                            loadingItem.visible = true;
                            console.log("DROPPED IN ITEM AREA: " + drag.keys);
                            console.log(dragItemIndex + " " + index);
                            const hlIndex = serviceItemList.currentIndex;
                            if (drag.keys[0] === "library") {
                                addItem(index,
                                        dragItemType,
                                        dragItemIndex);
                            } else if (drag.keys[0] === "serviceitem") {
                                /* ServiceItemModel.moveRows(serviceItemList.indexDragged, */
                                /*                       serviceItemList.moveToIndex, 1); */
                                /* serviceItemList.currentIndex = moveToIndex; */
                            }
                            dropHighlightLine.visible = false;
                            loadingItem.visible = false;
                        }

                        keys: ["library","serviceitem"]

                        Rectangle {
                            id: visServiceItem
                            width: serviceDrop.width
                            height: serviceDrop.height
                            anchors {
                                horizontalCenter: parent.horizontalCenter
                                verticalCenter: parent.verticalCenter
                            }
                            color: {
                                if (active)
                                    Kirigami.Theme.highlightColor;
                                else if (selected)
                                    Kirigami.Theme.focusColor;
                                else if (mouseHandler.containsMouse)
                                    Kirigami.Theme.hoverColor;
                                else
                                    Kirigami.Theme.backgroundColor;
                            }

                            Controls.Label {
                                id: label
                                anchors.left: dragHandle.right
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.leftMargin: 5
                                text: index + " " + name
                                elide: Text.ElideRight
                                width: parent.width - trailing.width - dragHandle.width - 25
                                color: {
                                    if (selected ||
                                        mouseHandler.containsMouse || active)
                                        Kirigami.Theme.highlightedTextColor;
                                    else
                                        Kirigami.Theme.textColor;
                                }
                            }

                            Kirigami.Icon {
                                id: trailing
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.rightMargin: 5
                                implicitWidth: Kirigami.Units.gridUnit
                                source: {
                                    switch (type) {
                                    case 'image': return "folder-pictures-symbolic";
                                    case 'video': return "folder-videos-symbolic";
                                    case 'song': return "folder-music-symbolic";
                                    case 'presentation': return "x-office-presentation-symbolic";
                                    default: return "slideshow-plugin";
                                    }
                                }
                                color: {
                                    if (selected ||
                                        mouseHandler.containsMouse || active)
                                        Kirigami.Theme.highlightedTextColor;
                                    else
                                        Kirigami.Theme.textColor;
                                }
                            }

                            /* onYChanged: serviceItemList.updateDrag(Math.round(y)); */

                            states: [
                                State {
                                    when: mouseHandler.drag.active
                                    ParentChange {
                                        target: visServiceItem
                                        parent: serviceItemList
                                    }

                                    PropertyChanges {
                                        target: visServiceItem
                                        /* backgroundColor: Kirigami.Theme.backgroundColor */
                                        /* textColor: Kirigami.Theme.textColor */
                                        anchors.verticalCenter: undefined
                                        anchors.horizontalCenter: undefined
                                    }
                                }
                            ]

                            /* Drag.dragType: Drag.Automatic */
                            /* Drag.active: mouseHandler.drag.active */
                            /* Drag.hotSpot.x: width / 2 */
                            /* Drag.hotSpot.y: height / 2 */
                            /* Drag.keys: ["serviceitem"] */

                            MouseArea {
                                id: mouseHandler
                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.LeftButton | Qt.RightButton

                                /* drag { */
                                /*     target: visServiceItem */
                                /*     axis: Drag.YAxis */
                                /*     /\* minimumY: root.y *\/ */
                                /*     /\* maximumY: serviceItemList.height - serviceDrop.height *\/ */
                                /*     smoothed: false */
                                /* } */

                                /* drag.onActiveChanged: { */
                                /*     if (mouseHandler.drag.active) { */
                                /*         serviceItemList.indexDragged = index; */
                                /*     }  */
                                /* } */

                                /* onPositionChanged: { */
                                /*     if (!pressed) { */
                                /*         return; */
                                /*     } */
                                /*     mouseArea.arrangeItem(); */
                                /* } */

                                onPressed: {
                                    serviceItemList.interactive = false;
                                }

                                onClicked: {
                                    if (mouse.button === Qt.RightButton) {
                                        if (!selected) {
                                            serviceItemList.currentIndex = index;
                                            ServiceItemModel.select(index);
                                        }
                                        rightClickMenu.popup(mouse);
                                    }
                                    else if ((mouse.button === Qt.LeftButton) &&
                                             (mouse.modifiers === Qt.ShiftModifier)) {
                                        ServiceItemModel.selectItems(index);
                                    } else {
                                        serviceItemList.currentIndex = index;
                                        ServiceItemModel.select(index);
                                    }
                                    refocusPresentation();
                                }

                                onDoubleClicked: {
                                    changeServiceItem(index);
                                }

                                onReleased: {
                                    console.log("should drop");
                                    visServiceItem.Drag.drop();
                                }
                            }

                            Presenter.DragHandle {
                                id: dragHandle
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.leftMargin: 5
                                /* width: 20 */
                                listItem: serviceListItem
                                listView: serviceItemList
                                onMoveRequested: ServiceItemModel.moveRows(oldIndex,
                                                                           newIndex,
                                                                           1)
                            }

                        }
                        Controls.Menu {
                            id: rightClickMenu
                            Kirigami.Action {
                                text: "Copy"
                            }
                            Kirigami.Action {
                                text: "Paste"
                            }
                            Kirigami.Action {
                                text: "Delete"
                                onTriggered: removeItem(index)
                            }

                            Controls.MenuSeparator {}

                            
                            Controls.Menu {
                                id: obsMenu
                                title: "Obs Scenes"
                                enabled: ObsModel.connected
                                Instantiator {
                                    model: ObsModel.scenes
                                    Kirigami.Action {
                                        text: modelData
                                        onTriggered: {
                                            showPassiveNotification("setting: " + modelData)
                                            ObsModel.setScene(modelData);
                                        }
                                    }
                                    onObjectAdded: obsMenu.insertAction(index, object)
                                    onObjectRemoved: obsMenu.removeAction(object)
                                }
                            }
                        }
                    }
                }
            }

            ListView {
                id: serviceItemList
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.left: parent.left
                width: serviceListScrollBar.visible ?
                    parent.width - serviceListScrollBar.width : parent.width
                clip: true
                spacing: 3
                property int indexDragged
                property int moveToIndex
                property int draggedY

                add: Transition {
                    NumberAnimation {properties: "width, height"; duration: 3000}
                    NumberAnimation { properties: "opacity"; duration: 3000 }
                }

                remove: Transition {
                    NumberAnimation { properties: "width, height"; duration: 3000 }
                    NumberAnimation { properties: "opacity"; duration: 3000 }
                }

                displaced: Transition {
                    NumberAnimation {properties: "x, y"; duration: 100}
                }

                model: ServiceItemModel

                delegate: Kirigami.DelegateRecycler {
                    width: serviceItemList.width
                    height: Kirigami.Units.gridUnit * 2
                    sourceComponent: delegate
                }
                Kirigami.WheelHandler {
                    id: wheelHandler
                    target: serviceItemList
                    filterMouseEvents: true
                    /* keyNavigationEnabled: true */
                }

                Controls.ScrollBar.vertical: Controls.ScrollBar {
                    id: serviceListScrollBar
                    parent: serviceItemList.parent
                    anchors.right: background.right
                    anchors.left: serviceItemList.right
                    anchors.top: serviceItemList.top
                    anchors.bottom: serviceItemList.bottom
                    active: hovered || pressed
                }

                /* function updateDrag(y) { */
                /*     if (moveToIndex === serviceItemList.indexAt(0,y)) */
                /*         return; */
                /*     else */
                /*         moveToIndex = serviceItemList.indexAt(0,y); */
                /*     moveRequested(indexDragged, moveToIndex); */
                /* } */

                /* function moveRequested(oldIndex, newIndex) { */
                /*     console.log("moveRequested: ", oldIndex, newIndex); */
                /*     ServiceItemModel.moveRows(oldIndex, newIndex, 1); */
                /*     indexDragged = newIndex; */
                /*     serviceItemList.currentIndex = newIndex; */
                /* } */

            }

            Rectangle {
                id: dropHighlightLine
                width: parent.width
                height: 4
                color: Kirigami.Theme.hoverColor
                visible: false
                Component.onCompleted: {
                    dragHighlightLine = dropHighlightLine;
                }
            }
            Canvas {
                x: dropHighlightLine.width - 8
                y: dropHighlightLine.y - 17
                z: 1
                width: 100; height: 100;
                contextType: "2d"
                renderStrategy: Canvas.Threaded
                onPaint: {
                    console.log(Kirigami.Theme.hoverColor);
                    var ctx = getContext("2d");
                    ctx.fillRule = Qt.OddEvenFill
                    ctx.fillStyle = Kirigami.Theme.hoverColor;
                    ctx.rotate(30);
                    ctx.transform(0.8, 0, 0, 0.8, 0, 30)
                    ctx.path = tearDropPath;
                    ctx.fill();
                }
                visible: dropHighlightLine.visible
            }
            Path {
                id: tearDropPath
                startX: dropHighlightLine.width
                startY: dropHighlightLine.y + 4
                PathSvg {
                    path: "M15 3
                           Q16.5 6.8 25 18
                           A12.8 12.8 0 1 1 5 18
                           Q13.5 6.8 15 3z"
                }
            }

            /* Shape { */
            /*     x: dropHighlightLine.width - 8 */
            /*     y: dropHighlightLine.y - 17 */
            /*     z: 1 */
            /*     width: 100; height: 100; */

            /*     ShapePath { */
            /*         fillColor: Kirigami.Theme.hoverColor */
            /*         startX: 0; startY: 0 */
            /*         PathLine { x: 180; y: 130 } */
            /*         PathLine { x: 20; y: 130 } */
            /*         PathLine { x: 20; y: 20 } */
            /*         PathArc { */
            /*             x: 40; y: 200; */
            /*             radiusX: 200; */
            /*             radiusY: 200; */
            /*             useLargeArc: true */
            /*         } */
            /*         PathLine { x: 40; y: 120 } */
            /*         PathArc { */
            /*             x: -40; y: 120; */
            /*             radiusX: 120; */
            /*             radiusY: 120; */
            /*             useLargeArc: true; */
            /*             direction: PathArc.Counterclockwise */
            /*         } */
            /*         PathLine { x: -40; y: 200 } */
            /*     } */
            /* } */

            Rectangle {
                id: loadingItem
                anchors.fill: parent
                color: Kirigami.Theme.backgroundColor
                visible: false

                Controls.BusyIndicator {
                    anchors.centerIn: parent
                    running: true
                }
            }
        } 

        Kirigami.ActionToolBar {
            id: serviceToolBar
            Layout.fillWidth: true
            opacity: 1.0
            display: Controls.Button.IconOnly
            actions: [
                Kirigami.Action {
                    text: "Up"
                    icon.name: "arrow-up"
                    onTriggered: {
                        const oldid = serviceItemList.currentIndex;
                        if (oldid <= 0)
                        {
                            showPassiveNotification("wow stop trying to go nego");
                            return;
                        }
                        const newid = serviceItemList.currentIndex - 1;
                        showPassiveNotification(oldid + " " + newid);
                        showPassiveNotification("Up");
                        const ans = ServiceItemModel.moveRows(oldid, newid, 1);
                        if (ans)
                        {
                            serviceItemList.currentIndex = newid;
                            showPassiveNotification("move was successful, newid: "
                                                    + serviceItemList.currentIndex);
                        }
                        else
                            showPassiveNotification("move was unsuccessful")
                    }
                },
                Kirigami.Action {
                    text: "Down"
                    icon.name: "arrow-down"
                    onTriggered: {
                        const id = serviceItemList.currentIndex;
                        if (id + 1 >= serviceItemList.count)
                        {
                            showPassiveNotification("wow you dummy you can't got further down");
                            return;
                        };
                        showPassiveNotification("moving ", id, " down");
                        const ans = ServiceItemModel.moveDown(id);
                        if (ans)
                        {
                            serviceItemList.currentIndex = id + 1;
                            showPassiveNotification("move was successful, newid: "
                                                    + serviceItemList.currentIndex);
                        }
                        else
                            showPassiveNotification("move was unsuccessful, id: "
                                                    + id);
                    }
                },
                Kirigami.Action {
                    text: "Remove"
                    icon.name: "delete"
                    onTriggered: {
                        showPassiveNotification("remove");
                        removeItems();
                    }
                },
                Kirigami.Action {
                    text: "Clear All"
                    icon.name: "list-remove-all"
                    onTriggered: {
                        showPassiveNotification("clearing all items");
                        ServiceItemModel.clearAll();
                        serviceItemList.forceLayout()
                    }
                },
                Kirigami.Action {
                    text: "Load"
                    icon.name: "fileopen"
                    onTriggered: {
                        loadingItem.visible = !loadingItem.visible;
                    }
                }
            ]
        }

    }

    Component.onCompleted: {
        /* totalServiceItems = serviceItemList.count; */
        console.log("THE TOTAL SERVICE ITEMS: " + totalServiceItems);
        activeServiceItem = serviceItemList.model.getItem(serviceItemList.currentIndex).name;
        // TODO
        // After getting the rust service_item_model setup let's add some properties that we can
        // use to find the active serviceItem and other things that will need
        // to see some global state of the model.
    }

    function removeItem(index) {
        ServiceItemModel.removeItem(index);
        serviceItemList.forceLayout()
        /* totalServiceItems--; */
    }

    function removeItems() {
        ServiceItemModel.removeItems();
        serviceItemList.forceLayout()
    }

    function addItem(index, type, itemIndex) {
        switch (type) {
        case 'image': {
            const image = imageProxyModel.getImage(itemIndex);
            console.log("adding: " + image.title + " of type " + type);
            ServiceItemModel.insertItem(index, image.title,
                                        "", type, image.filePath,
                                        "image", "",
                                        "", 0, 0, false, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        case 'video': {
            const video = videoProxyModel.getVideo(itemIndex);
            console.log("adding: " + video.title + " of type " + type);
            ServiceItemModel.insertItem(index, video.title,
                                        "", type, video.filePath,
                                        "video", "",
                                        "", 0, 0, video.loop, video.startTime, video.endTime);
            serviceItemList.forceLayout()
            return;
        }
        case 'song': {
            const lyrics = songProxyModel.getLyricList(itemIndex);
            const song = songProxyModel.getSong(itemIndex);
            /* showPassiveNotification(song.title); */
            console.log("adding: " + song.title +
                        " of type " + type +
                        " with " + lyrics.length + " slides");
            ServiceItemModel.insertItem(index, song.title,
                                        lyrics, type, song.background,
                                        song.backgroundType, 
                                        song.audio, song.font, song.fontSize,
                                        lyrics.length, true, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        case 'presentation': {
            const pres = presProxyModel.getPresentation(itemIndex);
            console.log("adding: " + pres.title +
                        " of type " + type +
                        " with " + pres.pageCount + " slides");
            ServiceItemModel.insertItem(index, pres.title,
                                        "", type, pres.filePath,
                                        "image",
                                        "", "", 0, pres.pageCount, false, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        default: return;
        }
        serviceItemList.forceLayout()
        /* totalServiceItems++; */
    }

    function appendItem(type, itemIndex) {
        switch (type) {
        case 'image': {
            const image = imageProxyModel.getImage(itemIndex);
            console.log("adding: " + image.title + " of type " + type);
            ServiceItemModel.addItem(image.title,
                                     type, image.filePath,
                                     "image", "", "",
                                     "", 0, 0, false, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        case 'video': {
            const video = videoProxyModel.getVideo(itemIndex);
            console.log("adding: " + video.title + " of type " + type);
            ServiceItemModel.addItem(video.title,
                                     type, video.filePath,
                                     "video", "", "",
                                     "", 0, 0, video.loop, video.startTime, video.endTime);
            serviceItemList.forceLayout()
            return;
        }
        case 'song': {
            const lyrics = songProxyModel.getLyricList(itemIndex);
            const song = songProxyModel.getSong(itemIndex);
            console.log("adding: " + song.title +
                        " of type " + type +
                        " with " + lyrics.length + " slides");
            ServiceItemModel.addItem(song.title,
                                     type, song.background,
                                     song.backgroundType, lyrics,
                                     song.audio, song.font, song.fontSize,
                                     lyrics.length, true, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        case 'presentation': {
            const pres = presProxyModel.getPresentation(itemIndex);
            console.log("adding: " + pres.title +
                        " of type " + type +
                        " with " + pres.pageCount + " slides");
            ServiceItemModel.addItem(pres.title,
                                     type, pres.filePath,
                                     "image", "",
                                     "", "", 0, pres.pageCount,
                                     false, 0.0, 0.0);
            serviceItemList.forceLayout()
            return;
        }
        default: return;
        }
        serviceItemList.forceLayout()
    }

}
