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
        id: listView

        model: TimelineModel {
            connection: root.connection
            roomId: root.roomId
        }
        // verticalLayoutDirection: ListView.BottomToTop

        delegate: QQC2.ItemDelegate {
            required property string eventId
            required property string body
            required property int index

            width: parent.width
            text: (listView.count - index - 1) + " " + body
        }
    }
}
