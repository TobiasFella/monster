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

    Connection {
        id: connection
        Component.onCompleted: {
            RoomAvatarImageProvider.connection = connection;
            connection.restore();
        }
        onLoggedInChanged: {
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
            FormCard.FormTextFieldDelegate {
                id: matrixIdField
                label: i18nc("@action:textfield", "Matrix Id")
            }
            FormCard.FormTextFieldDelegate {
                id: passwordField
                label: i18nc("@action:textfield", "Password")
                echoMode: QQC2.TextField.Password
            }
            FormCard.FormButtonDelegate {
                text: i18nc("@action:button", "Login")
                onClicked: connection.login(matrixIdField.text, passwordField.text)
            }
        }
    }
}
