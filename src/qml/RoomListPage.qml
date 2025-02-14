// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts

import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.formcard as FormCard

import im.arctic.monster

Kirigami.ScrollablePage {
    id: root

    title: i18nc("@title", "Rooms")

    required property Connection connection

    Connections {
        target: root.connection
        function onOpenRoom(): void {
            room => pageStack.push(Qt.createComponent("im.arctic.monster", "RoomPage"), {
                roomId: room,
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
