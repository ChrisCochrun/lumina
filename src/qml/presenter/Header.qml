import QtQuick 2.13
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Window 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter

Kirigami.ActionToolBar {
    id: root
    alignment: Qt.AlignRight
    height: Kirigami.Units.gridUnit * 2.0

    Kirigami.Heading {
        text: "Lumina"
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: 20
    }

    actions: [

        Kirigami.Action {
            displayComponent: Component {
                Kirigami.SearchField {
                    id: searchField
                    anchors.centerIn: parent
                    width: parent.width / 3
                    onAccepted: showPassiveNotification(searchField.text, 3000)
                    background: Presenter.TextBackground {
                        control: searchField
                    }
                }
            }
        },

        Kirigami.Action {
            icon.name: editMode ? "view-preview" : "edit"
            text: editMode ? "Preview" : "Edit"
            onTriggered: toggleEditMode()
        },
        
        Kirigami.Action {
            icon.name: "view-presentation"
            text: presenting ? "Presenting" : "Go Live" 
            onTriggered: {
                console.log("Window is loading");
                togglePresenting();
            }
        },

        Kirigami.Action {
            icon.name: libraryOpen ? "sidebar-collapse-right" : "sidebar-expand-right"
            text: libraryOpen ? "Close Library" : "Open Library"
            onTriggered: toggleLibrary()
        }

    ]
}
