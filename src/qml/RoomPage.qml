// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2

import org.kde.kirigami as Kirigami

import im.arctic.monster

Kirigami.ScrollablePage {
    id: root

    property string roomId

    Component.onCompleted: TimelineModel.roomId = root.roomId
    title: i18nc("@title", "Room")

    ListView {
        model: TimelineModel
        delegate: QQC2.ItemDelegate {
            width: root.width
            required property string eventId
            text: eventId
        }
    }
}
