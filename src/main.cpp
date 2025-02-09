// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

#include <KLocalizedString>

#include <QThread>
#include <QGuiApplication>
#include <QQmlContext>
#include <QQmlApplicationEngine>
#include <QApplication>

#include "roomavatarimageprovider.h"

using namespace Qt::Literals::StringLiterals;

int main(int argc, char *argv[])
{
    KLocalizedString::setApplicationDomain("monster");
    QApplication app(argc, argv);

    QGuiApplication::setApplicationName("Monster"_L1);
    QGuiApplication::setApplicationDisplayName("Monster"_L1);
    QGuiApplication::setOrganizationName("Arctic"_L1);
    QGuiApplication::setOrganizationDomain("arctic.im"_L1);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextObject(new KLocalizedContext(&engine));
    engine.loadFromModule("im.arctic.monster", "Main");
    engine.addImageProvider(QStringLiteral("roomavatar"), RoomAvatarImageProvider::instance());

    return app.exec();
}
