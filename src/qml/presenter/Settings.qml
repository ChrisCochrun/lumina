import QtQuick 2.13
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0
import Qt.labs.settings 1.0

Kirigami.OverlaySheet {
    id: root
    property ListModel theModel

    header: Kirigami.Heading {
        text: "Settings"
    }

    Component.onCompleted: {
        /* ObsModel.getObs(); */
        /* ObsModel.updateScenes(); */
    }

    Kirigami.FormLayout {
        implicitHeight: Kirigami.Units.gridUnit * 30
	Controls.ComboBox {
	    id: screenSelectionField
	    Kirigami.FormData.label: i18nc("@label:textbox", "Presentation Screen:")
            model: screens
            textRole: "name"
	    onActivated: {
                presentationScreen = screens[currentIndex];
                console.log(PresWindow.screen);
                PresWindow.screen = screens[currentIndex];
                console.log(PresWindow.screen);
            }

            popup: Controls.Popup {
                y: screenSelectionField.height + 10
                z: 1000
                width: screenSelectionField.width
                implicitHeight: contentItem.implicitHeight
                padding: 1

                contentItem: ListView {
                    clip: true
                    implicitHeight: contentHeight
                    model: screenSelectionField.popup.visible ? screenSelectionField.delegateModel : null
                    currentIndex: screenSelectionField.highlightedIndex

                    Controls.ScrollIndicator.vertical: Controls.ScrollIndicator { }
                }

                background: Rectangle {
                    border.color: Kirigami.Theme.hoverColor
                    radius: 2
                }
            }
	}
        Controls.ToolButton {
            id: soundEffectBut
            Kirigami.FormData.label: i18nc("@label:button", "Sound Effect:")
            text: "Sound Effect"
            onClicked: soundFileDialog.open()
        }

        Controls.ToolButton {
            Kirigami.FormData.label: i18nc("@label:button", "OBS debug")
            text: "Obs Debug"
            onClicked: {
                ObsModel.getObs();
                ObsModel.updateScenes();
                console.log(ObsModel.scenes);
            }
        }

        Kirigami.ActionTextField {
            Kirigami.FormData.label: i18nc("@label:textbox", "Obs Connection")
            text: ObsModel.connected
        }
        
    }

}
