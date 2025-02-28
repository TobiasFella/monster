// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QtQmlIntegration/qqmlintegration.h>

#include "room.h"

#include "quotient_export.h"

namespace Quotient
{
class RoomStream;

class QUOTIENT_EXPORT Connection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

public:
    ~Connection();

    QString matrixId() const;

    Q_INVOKABLE void open(const QString &roomId);
    Q_INVOKABLE void logout();

    Q_INVOKABLE void createRoom(const QString &name = {}, const QString &topic = {}, const QString &alias = {});

    Q_INVOKABLE Quotient::Room *room(const QString &id);

    /*
     * @brief Get a room stream for the connection.
     *
     * You own this, look after it.
     */
    std::unique_ptr<RoomStream> roomStream();

    void roomAvatar(const QString &roomId);

    class Private;
    std::unique_ptr<Private> d;
Q_SIGNALS:
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void openRoom(Room *room);
    void loggedOut();

private:
    friend class PendingConnection;

    Connection(std::unique_ptr<Private> d);
    static Connection *loginWithPassword(const QString &matrixId, const QString &password);
    static Connection *loadAccount(const QString &matrixId);
};

}
