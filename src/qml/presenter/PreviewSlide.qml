import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import QtMultimedia 5.15
/* import QtAudioEngine 1.15 */
import QtGraphicalEffects 1.15
import org.kde.kirigami 2.13 as Kirigami
import "./" as Presenter
import org.presenter 1.0

Item {
    id: root

    // These properties are for the slides visuals
    property real textSize: 50
    property bool dropShadow: false
    property url imageSource
    property int pdfIndex
    property string chosenFont: "Quicksand"
    property string text
    property color backgroundColor
    property var hTextAlignment: Text.AlignHCenter
    property var vTextAlignment: Text.AlignVCenter

    // These properties help to determine the state of the slide
    property string itemType

    implicitWidth: 1920
    implicitHeight: 1080

    Rectangle {
        id: basePrColor
        anchors.fill: parent
        color: "black"

        Image {
            id: backgroundImage
            anchors.fill: parent
            source: imageSource
            fillMode: itemType == "song" ? Image.PreserveAspectCrop : Image.PreserveAspectFit
            clip: true
            visible: true
            currentFrame: pdfIndex
        }

        Presenter.LoadingSpinner {
            id: loadingSpinner
            color: Kirigami.Theme.highlightColor
            running: !fileHelper.validate(imageSource)
            anchors.fill: parent
        }

        FastBlur {
            id: imageBlue
            anchors.fill: parent
            source: backgroundImage
            radius: blurRadius

            Controls.Label {
                id: lyrics
                text: root.text
                /* text: root.width / textSize */
                font.pixelSize: root.width / 1000 * root.textSize 
                /* minimumPointSize: 5 */
                fontSizeMode: Text.Fit
                font.family: chosenFont
                horizontalAlignment: hTextAlignment
                verticalAlignment: vTextAlignment
                style: Text.Raised
                wrapMode: Text.WordWrap
                anchors.fill: parent
                anchors.topMargin: 10
                anchors.bottomMargin: 10
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                clip: true

                layer.enabled: true
                layer.effect: DropShadow {
                    horizontalOffset: 5
                    verticalOffset: 5
                    radius: 11.0
                    samples: 24
                    color: "#80000000"
                }
            }
        }
    }
}
