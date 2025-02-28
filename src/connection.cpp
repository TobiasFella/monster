// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "connection.h"

#include <QDebug>

#include <memory>
#include <qt6keychain/keychain.h>

#include "roomstream.h"
#include "connection_p.h"
#include "dispatcher.h"
#include "lib.rs.h"
#include "room.h"
#include "room_p.h"
#include "utils.h"

using namespace Qt::Literals::StringLiterals;
using namespace Quotient;

Connection::~Connection() = default;

Connection::Connection(std::unique_ptr<Private> d)
    : QObject(nullptr)
    , d(std::move(d))
{
}

QString Connection::matrixId() const
{
    return stringFromRust((*d->wrapper->m_connection)->matrix_id());
}

void Connection::open(const QString &roomId)
{
    Q_EMIT openRoom(room(roomId));
}

rust::Box<sdk::Connection> &Connection::Private::connection() const
{
    return *wrapper->m_connection;
}

void Connection::logout()
{
    connect(Dispatcher::instance(), &Dispatcher::loggedOut, this, [this](const QString &matrixId) {
        if (matrixId != this->matrixId()) {
            return;
        }
        Q_EMIT loggedOut();
        deleteLater();
    });
    (*d->wrapper->m_connection)->logout();
}

void Connection::createRoom(const QString &name, const QString &topic, const QString &alias)
{
    auto options = sdk::room_create_options_new();
    if (!name.isEmpty()) {
        options->set_name(stringToRust(name));
    }

    if (!topic.isEmpty()) {
        options->set_topic(stringToRust(topic));
    }

    if (!alias.isEmpty()) {
        options->set_room_alias(stringToRust(alias));
    }
    (*d->wrapper->m_connection)->create_room(*options);
}

Room *Connection::room(const QString &id)
{
    // TODO cache room objects
    return new Room(std::make_unique<Room::Private>(new RustRoomWrapper((*d->wrapper->m_connection)->room(stringToRust(id)))));
}

void Connection::roomAvatar(const QString &roomId)
{
    (*d->wrapper->m_connection)->room_avatar(stringToRust(roomId));
}

std::unique_ptr<RoomStream> Connection::roomStream()
{
    return std::make_unique<RoomStream>(this);
}
