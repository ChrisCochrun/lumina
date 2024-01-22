import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0
import mpv 1.0

ListView {
    // The active items X value from root
    property int activeX
    id: previewSlidesList
    anchors.bottom: parent.bottom
    width: parent.width
    height: Kirigami.Units.gridUnit * 9
    orientation: ListView.Horizontal
    spacing: Kirigami.Units.smallSpacing * 2
    cacheBuffer: 900
    reuseItems: true
    model: SlideModel
    delegate: Presenter.PreviewSlideListDelegate {}
    highlightFollowsCurrentItem: false

    Kirigami.WheelHandler {
        id: wheelHandler
        target: previewSlidesList
        filterMouseEvents: true
    }

    Controls.ScrollBar.horizontal: Controls.ScrollBar {
        active: hovered || pressed
    }

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


    Connections {
        target: SlideModel
        function onActiveChanged(index) {
            console.log("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
            console.log(index);
            previewSlidesList.currentIndex = index;
            previewSlidesList.positionViewAtIndex(index, ListView.Center);
            currentSlide = index;
            const serviceItemId = SlideModel.getItem(index).serviceItemId;
            console.log(serviceItemId);
            currentServiceItem = serviceItemId;
        }
    }
}
