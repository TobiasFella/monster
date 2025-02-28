// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2

import org.kde.kirigami as Kirigami

import org.kde.quotient.demo

Kirigami.Page {
    id: root

    required property Room room
    required property Connection connection

    title: room.displayName

    padding: 0

    QQC2.ScrollView {
        anchors.fill: parent
        anchors.margins: 0
        clip: true
        ListView {
            id: listView

            model: ReversedTimelineModel {
                sourceModel: TimelineModel {
                    id: timelineModel
                    connection: root.connection
                    room: root.room
                }
            }
            verticalLayoutDirection: ListView.BottomToTop

            delegate: QQC2.ItemDelegate {
                required property string eventId
                required property string body
                required property int index
                required property string timestamp

                // width: parent.width
                text: timestamp + " " + body
            }
        }
    }

    QQC2.TextArea {
        id: textArea
        placeholderText: "Message..."
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 0
        height: 50

        QQC2.Button {
            icon.name: "document-send"
            anchors.right: textArea.right
            onClicked: timelineModel.sendMessage(textArea.text)
        }
    }
}
