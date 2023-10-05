import QtQuick 2.13
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Labs
import QtQuick.Pdf 5.15
import QtQml.Models 2.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

ColumnLayout {
    id: root
    property var proxyModel
    property var innerModel
    property string libraryType
    property string headerLabel
    property string itemLabel
    property string itemSubtitle
    property string itemIcon
    property string count
    property var newItemFunction
    property var deleteItemFunction
    property ListView libraryList: libraryList

    states: [
        State {
            name: "deselected"
            when: (selectedLibrary !== libraryType)
            PropertyChanges {
                target: root
                Layout.preferredHeight: Kirigami.Units.gridUnit * 1.5
            }
        },
        State {
            name: "selected"
            when: (selectedLibrary == libraryType)
            PropertyChanges { target: root }
        }
    ]

    transitions: Transition {
        to: "*"
        NumberAnimation {
            target: root
            properties: "preferredHeight"
            easing.type: Easing.OutCubic
            duration: 300
        }
    }

    Rectangle {
        id: libraryPanel
        Layout.preferredHeight: 40
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignTop
        z: 2
        color: Kirigami.Theme.backgroundColor

        Controls.Label {
            id: libraryLabel
            /* anchors.centerIn: parent */
            anchors.left: parent.left
            anchors.leftMargin: 15
            anchors.verticalCenter: parent.verticalCenter
            elide: Text.ElideLeft
            text: headerLabel
            color: libraryMouseArea.containsMouse ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor
        }

        Controls.Label {
            id: countLabel
            anchors {left: libraryLabel.right
                     verticalCenter: libraryLabel.verticalCenter
                     leftMargin: 15}
            text: count
            font.pointSize: 9
            color: libraryMouseArea.containsMouse ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor
        }

        Kirigami.Icon {
            id: drawerArrow
            anchors {right: parent.right
                     verticalCenter: libraryLabel.verticalCenter
                     rightMargin: 10}
            source: "arrow-down"
            rotation: selectedLibrary == libraryType ? 0 : 180
            color: libraryMouseArea.containsMouse ? Kirigami.Theme.focusColor : Kirigami.Theme.textColor

            Behavior on rotation {
                NumberAnimation {
                    easing.type: Easing.OutCubic
                    duration: 300
                }
            }
        }

        MouseArea {
            id: libraryMouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: {
                if (selectedLibrary == libraryType)
                    selectedLibrary = ""
                else
                    selectedLibrary = libraryType
                /* console.log(selectedLibrary) */
            }
        }
    }

    Rectangle {
        id: libraryHeader
        z: 2
        Layout.preferredHeight: 40
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignTop
        /* width: parent.width */
        color: Kirigami.Theme.backgroundColor
        opacity: 1
        state: "selected"

        states: [
            State {
                name: "deselected"
                when: (selectedLibrary !== libraryType)
                PropertyChanges {
                    target: libraryHeader
                    Layout.preferredHeight: 0
                }
            },
            State {
                name: "selected"
                when: (selectedLibrary == libraryType)
                PropertyChanges { target: libraryHeader }
            }
        ]

        Kirigami.ActionToolBar {
            height: parent.height
            width: parent.width
            display: Controls.Button.IconOnly
            visible: selectedLibrary == libraryType
            rightPadding: 5
            actions: [
                Kirigami.Action {
                    icon.name: "document-new"
                    text: "New " + libraryType
                    tooltip: "Add a new " + libraryType
                    onTriggered: newItemFunction()
                },
                
                Kirigami.Action {
                    id: searchField
                    displayComponent: Kirigami.SearchField {
                        id: searchField
                        height: parent.height
                        width: parent.width - 40
                        onAccepted: proxyModel.setFilterRegularExpression(searchField.text)
                        background: Presenter.TextBackground {
                            control: searchField
                        }
                    }
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
        Layout.fillHeight: true
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignTop
        id: libraryList
        model: proxyModel
        clip: true
        ItemSelectionModel {
            id: selectionModel
            model: proxyModel
            onSelectionChanged: {
                /* showPassiveNotification("deslected: " + deselected); */
                /* showPassiveNotification("selected: " + selected); */
                /* console.log(selected); */
            }
        }
        delegate: libraryDelegate
        state: "selected"

        states: [
            State {
                name: "deselected"
                when: (selectedLibrary !== libraryType)
                PropertyChanges {
                    target: libraryList
                    Layout.preferredHeight: 0
                }
            },
            State {
                name: "selected"
                when: (selectedLibrary == libraryType)
                PropertyChanges { target: libraryList }
            }
        ]

        transitions: Transition {
            to: "*"
            NumberAnimation {
                target: libraryList
                properties: "preferredHeight"
                easing.type: Easing.OutCubic
                duration: 300
            }
        }

        Component {
            id: libraryDelegate
            Item{
                implicitWidth: ListView.view.width
                height: selectedLibrary == libraryType ? 50 : 0
                Kirigami.BasicListItem {
                    id: listItem

                    property bool rightMenu: false
                    property bool selected: selectionModel.isSelected(proxyModel.idx(index))
                    property bool fileValidation: {
                        if (filePath)
                            fileHelper.validate(filePath)
                        else
                            false
                    }

                    implicitWidth: libraryList.width
                    height: selectedLibrary == libraryType ? 50 : 0
                    clip: true
                    label: title
                    subtitle: {
                        if (libraryType == "song")
                            author
                        else if (fileValidation)
                            filePath;
                        else
                            "file is missing"
                    }
                    icon: itemIcon
                    iconSize: Kirigami.Units.gridUnit
                    supportsMouseEvents: false
                    backgroundColor: Kirigami.Theme.backgroundColor;
                    Binding on backgroundColor {
                        when: dragHandler.containsMouse ||
                            (selectionModel.hasSelection &&
                             selectionModel.isSelected(proxyModel.idx(index)))
                        value: Kirigami.Theme.highlightColor
                    }

                    textColor: {
                        if (selectedLibrary == "song")
                            Kirigami.Theme.textColor;
                        else if (fileValidation) {
                            Kirigami.Theme.textColor;
                        }
                        else
                            "red"
                    }

                    Binding on textColor {
                        when: dragHandler.containsMouse ||
                            (selectionModel.hasSelection &&
                             selectionModel.isSelected(proxyModel.idx(index)))
                        value: Kirigami.Theme.highlightedTextColor
                    }

                    Behavior on height {
                        NumberAnimation {
                            easing.type: Easing.OutCubic
                            duration: 300
                        }
                    }
                    Drag.active: dragHandler.drag.active
                    Drag.hotSpot.x: width / 2
                    Drag.hotSpot.y: height / 2
                    Drag.keys: [ "library" ]

                    states: State {
                        name: "dragged"
                        when: listItem.Drag.active
                        PropertyChanges {
                            target: listItem
                            x: x
                            y: y
                            width: width
                            height: height
                        }
                        ParentChange {
                            target: listItem
                            parent: rootApp.overlay
                        }
                    }
                }

                MouseArea {
                    id: dragHandler
                    anchors.fill: parent
                    hoverEnabled: true
                    drag {
                        target: listItem
                        onActiveChanged: {
                            if (dragHandler.drag.active) {
                                dragItemIndex = index;
                                dragItemType = libraryType;
                                draggedLibraryItem = self;
                            } else {
                                listItem.Drag.drop();
                                dragHighlightLine.visible = false;
                            }
                        }
                        filterChildren: true
                        threshold: 10
                        /* onDropped: dragHighlightLine.visible = false; */
                    }
                    MouseArea {
                        id: clickHandler
                        anchors.fill: parent
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: {
                            if (mouse.button === Qt.RightButton) {
                                if(selectionModel.selectedIndexes.length <= 1)
                                    selectionModel.select(proxyModel.idx(index),
                                                          ItemSelectionModel.ClearAndSelect);
                                rightClickMenu.popup()
                            }
                            else if ((mouse.button === Qt.LeftButton) &&
                                     (mouse.modifiers === Qt.ShiftModifier)) {
                                if (libraryList.currentIndex < index) {
                                    for (let i = libraryList.currentIndex; i <= index; i++) {
                                        selectionModel.select(proxyModel.idx(i),
                                                              ItemSelectionModel.Select);
                                    }
                                }
                                else {
                                    for (let i = index; i <= libraryList.currentIndex; i++) {
                                        selectionModel.select(proxyModel.idx(i),
                                                              ItemSelectionModel.Select);
                                    }
                                }
                                console.log(selectionModel.selectedIndexes);
                            } else {
                                selectionModel.select(proxyModel.idx(index),
                                                      ItemSelectionModel.ClearAndSelect);
                                libraryList.currentIndex = index;
                            }
                        }
                        onDoubleClicked: {
                            libraryList.currentIndex = index;
                            if (!editMode)
                                editMode = true;
                            editSwitch(index, libraryType);
                        }

                    }
                }
                Controls.Menu {
                    id: rightClickMenu
                    x: clickHandler.mouseX
                    y: clickHandler.mouseY + 10
                    Kirigami.Action {
                        text: "delete"
                        onTriggered: {
                            var selection = [];
                            var length = selectionModel.selectedIndexes.length;
                            for (let i = 0; i < length; i++) {
                                selection.push(selectionModel.selectedIndexes[i].row);
                            }
                            root.deleteItemFunction(selection);
                        }
                    }
                }
            }
        }

        Kirigami.WheelHandler {
            id: wheelHandler
            target: libraryList
            filterMouseEvents: true
            keyNavigationEnabled: true
        }

        Controls.ScrollBar.vertical: Controls.ScrollBar {
            active: hovered || pressed
        }

        function selectItems(row) {
            let currentRow = selectionModel.selectedIndexes[0].row;
            if (row === currentRow)
                return;

            if (row > currentRow) {
                for (var i = currentRow; i <= row; i++) {
                    let idx = proxyModel.idx(i);
                    selectionModel.select(idx, ItemSelectionModel.Select);
                }
            }
            else {
                for (var i = row; i <= currentRow; i++) {
                    let idx = proxyModel.idx(i);
                    selectionModel.select(idx, ItemSelectionModel.Select);
                }
            }
        }
    }
}
