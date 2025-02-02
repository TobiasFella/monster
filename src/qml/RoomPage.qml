// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2

import org.kde.kirigami as Kirigami

import im.arctic.monster

Kirigami.ScrollablePage {
    id: root

    required property string roomId
    required property Connection connection

    title: i18nc("@title", "Room")

    ListView {
        anchors.fill: parent
        model: TimelineModel {
            connection: root.connection
            roomId: root.roomId
        }
        delegate: QQC2.ItemDelegate {
            required property string eventId

            width: root.width
            text: eventId
        }
    }
}
