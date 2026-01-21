// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts

import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.formcard as FormCard

import im.arctic.monster

Kirigami.ApplicationWindow {
    id: root

    width: 800
    height: 600

    title: i18nc("@title:window", "Monster")

    Accounts {
        id: accounts
    }

    Connections {
        id: pendingConnections

        ignoreUnknownSignals: true

        function onReady() {
            const connection = (target as PendingConnection).connection();
            RoomAvatarImageProvider.connection = connection;
            root.pageStack.pop();
            root.pageStack.push(Qt.createComponent("im.arctic.monster", "RoomListPage"), {
                connection: connection
            }, {});
        }
    }

    pageStack.initialPage: FormCard.FormCardPage {
        title: i18nc("@title", "Login")
        FormCard.FormHeader {
            title: i18nc("@title", "Login")
        }
        FormCard.FormCard {
            // FormCard.FormTextFieldDelegate {
            //     id: matrixIdField
            //     label: i18nc("@action:textfield", "Matrix Id")
            // }
            // FormCard.FormTextFieldDelegate {
            //     id: passwordField
            //     label: i18nc("@action:textfield", "Password")
            //     echoMode: QQC2.TextField.Password
            // }
            FormCard.FormTextFieldDelegate {
                id: serverNameField
                label: i18nc("@action:textfield", "Server address")
            }
            FormCard.FormButtonDelegate {
                text: i18nc("@action:button", "Login")
                onClicked: pendingConnections.target = accounts.loginWithOidc(serverNameField.text)
            }
        }
        FormCard.FormCard {
            Repeater {
                model: accounts.availableAccounts
                delegate: FormCard.FormButtonDelegate {
                    required property string modelData
                    text: modelData
                    onClicked: pendingConnections.target = accounts.loadAccount(modelData)
                }
            }
        }
    }
}
