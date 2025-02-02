// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

#include <KLocalizedString>

#include <QThread>
#include <QGuiApplication>
#include <QQmlContext>
#include <QQmlApplicationEngine>
#include <QApplication>

#include "roomavatarimageprovider.h"

int main(int argc, char *argv[])
{
    KLocalizedString::setApplicationDomain("monster");
    QApplication app(argc, argv);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextObject(new KLocalizedContext(&engine));
    engine.loadFromModule("im.arctic.monster", "Main");
    engine.addImageProvider(QStringLiteral("roomavatar"), &RoomAvatarImageProvider::instance());

    return app.exec();
}
