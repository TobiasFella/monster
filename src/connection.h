// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QQmlEngine>
#include <qqmlintegration.h>

#include "ffi.rs.h"
#include "room.h"
#include "sdk/src/task.h"

namespace Quotient
{
class RoomStream;

class Connection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

    Q_PROPERTY(QString matrixId READ matrixId CONSTANT)

public:
    ~Connection() override;

    [[nodiscard]] rust::Box<sdk::Connection> &connection() const;
    [[nodiscard]] QString matrixId() const;

    Q_INVOKABLE void open(const QString &roomId);
    Q_INVOKABLE void logout();

    Q_INVOKABLE void createRoom(const QString &name = {}, const QString &topic = {}, const QString &alias = {});

    Q_INVOKABLE Quotient::Room *room(const QString &id);
    Q_INVOKABLE bool hasRoom(const QString &id);

    Q_INVOKABLE Task *setDisplayName(const QString &displayName);

    /*
     * @brief Get a room stream for the connection.
     *
     * You own this, look after it.
     */
    std::unique_ptr<RoomStream> roomStream();

Q_SIGNALS:
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void openRoom(Quotient::Room *room);
    void loggedOut();

private:
    class Private;
    std::unique_ptr<Private> d;
    friend class PendingConnection;

    explicit Connection(std::optional<rust::Box<sdk::Connection>> wrapper);
};

}
