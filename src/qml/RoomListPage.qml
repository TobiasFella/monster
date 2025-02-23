// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import Qt.labs.qmlmodels

import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.formcard as FormCard
import org.kde.kirigamiaddons.delegates as Delegates

import im.arctic.monster

Kirigami.ScrollablePage {
    id: root

    title: i18nc("@title", "Rooms")

    required property Connection connection

    actions: [
        Kirigami.Action {
            text: i18nc("@action:button", "Log out")
            onTriggered: root.connection.logout()
        },
        Kirigami.Action {
            text: i18nc("@action:button", "Create Room")
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

    TreeView {
        id: treeView
        topMargin: Math.round(Kirigami.Units.smallSpacing / 2)

        clip: true
        reuseItems: false

        model: SortFilterRoomTreeModel {
            sourceModel: RoomTreeModel {
                connection: root.connection
            }
        }

        selectionModel: ItemSelectionModel {}

        delegate: DelegateChooser {
            role: "delegateType"

            DelegateChoice {
                roleValue: "section"
                delegate: RoomTreeSection {
                    collapsed: root.collapsed
                }
            }

            DelegateChoice {
                roleValue: "normal"
                delegate: RoomDelegate {
                    id: roomDelegate
                    required property int row
                    required property TreeView treeView
                    required property bool current
                    onCurrentChanged: if (current) {
                        forceActiveFocus(Qt.TabFocusReason);
                    }

                    implicitWidth: treeView.width
                    connection: root.connection
                    collapsed: root.collapsed
                    highlighted: RoomManager.currentRoom === currentRoom
                }
            }

            DelegateChoice {
                roleValue: "addDirect"
                delegate: Delegates.RoundedItemDelegate {
                    text: i18n("Find your friends")
                    icon.name: "list-add-user"
                    icon.width: Kirigami.Units.gridUnit * 2
                    icon.height: Kirigami.Units.gridUnit * 2

                    onClicked: pageStack.pushDialogLayer(Qt.createComponent('org.kde.neochat', 'UserSearchPage'), {
                        connection: root.connection
                    }, {
                        title: i18nc("@title", "Find your friends")
                    })
                }
            }
        }
    }
}
