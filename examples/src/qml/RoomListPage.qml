// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts

import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.formcard as FormCard

import org.kde.quotient.demo

Kirigami.ScrollablePage {
    id: root

    title: qsTr("Rooms")

    required property Connection connection

    actions: [
        Kirigami.Action {
            text: qsTr("Log out")
            onTriggered: root.connection.logout()
        },
        Kirigami.Action {
            text: qsTr("Create Room")
            onTriggered: root.connection.createRoom("Hello", "World", "")
        }
    ]

    Connections {
        target: root.connection
        function onOpenRoom(): void {
            room => pageStack.push(Qt.createComponent("im.arctic.monster", "RoomPage"), {
                room: room,
                connection: connection,
            });
        }
    }

    ListView {
        model: RoomsModel {
            connection: root.connection
        }
        delegate: QQC2.ItemDelegate {
            id: roomDelegate
            width: root.width
            required property string roomId
            required property string displayName
            required property string avatarUrl
            text: roomDelegate.displayName
            icon.source: roomDelegate.avatarUrl
            onClicked: root.connection.open(roomDelegate.roomId)
        }
    }
}
