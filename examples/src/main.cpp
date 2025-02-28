// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

#include <QGuiApplication>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QThread>

#include "roomavatarimageprovider.h"

using namespace Qt::Literals::StringLiterals;

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    QGuiApplication::setApplicationName("QuotientDemo"_L1);
    QGuiApplication::setApplicationDisplayName("QuotientDemo"_L1);
    QGuiApplication::setOrganizationName("Quotient"_L1);
    QGuiApplication::setOrganizationDomain("org.kde.quotient"_L1);

    QQmlApplicationEngine engine;
    engine.loadFromModule("org.kde.quotient.demo", "Main");
    engine.addImageProvider(QStringLiteral("roomavatar"), RoomAvatarImageProvider::instance());

    return app.exec();
}
