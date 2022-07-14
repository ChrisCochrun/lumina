import QtQuick 2.13
import QtQuick.Dialogs 1.0
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.2
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Kirigami.OverlaySheet {

    property ListModel theModel

    id: root
    header: Kirigami.Heading {
        text: "Settings"
    }

    /* Component.onCompleted: { */
    /*     showPassiveNotification(screenModel.get(1).name) */
    /* } */

    Kirigami.FormLayout {
	Controls.ComboBox {
	    id: screenSelectionField
	    Kirigami.FormData.label: i18nc("@label:textbox", "Presentation Screen:")
            model: screens
            textRole: "name"
	    onActivated: {
                presentationScreen = screens[currentIndex];
            }
	}
    }
}