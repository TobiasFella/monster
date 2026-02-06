// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QQmlEngine>
#include <qqmlintegration.h>

#include "lib.rs.h"
#include "room.h"

namespace Quotient
{
class RoomStream;

struct RustConnectionWrapper
{
    std::optional<rust::Box<sdk::Connection>> m_connection;
};

class Connection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

public:
    ~Connection();

    rust::Box<sdk::Connection> &connection() const;
    QString matrixId() const;

    Q_INVOKABLE void open(const QString &roomId);
    Q_INVOKABLE void logout();

    Q_INVOKABLE void createRoom(const QString &name = {}, const QString &topic = {}, const QString &alias = {});

    Q_INVOKABLE Quotient::Room *room(const QString &id);
    Q_INVOKABLE bool hasRoom(const QString &id);

    /*
     * @brief Get a room stream for the connection.
     *
     * You own this, look after it.
     */
    std::unique_ptr<RoomStream> roomStream();

Q_SIGNALS:
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void openRoom(Room *room);
    void loggedOut();

private:
    class Private;
    std::unique_ptr<Private> d;
    friend class PendingConnection;

    Connection(RustConnectionWrapper *wrapper);

    static Connection *loginWithPassword(const QString &matrixId, const QString &password);
    static Connection *loadAccount(const QString &matrixId);
};

}
