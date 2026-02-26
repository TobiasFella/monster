// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "connection.h"

#include <QDebug>
#include <QGuiApplication>

#include <memory>
#include <qt6keychain/keychain.h>

#include "dispatcher.h"
#include "ffi.rs.h"
#include "room.h"
#include "roomstream.h"
#include "sdk/src/task.h"
#include "utils.h"

using namespace Qt::Literals::StringLiterals;
using namespace Quotient;

class Connection::Private
{
public:
    std::optional<rust::Box<sdk::Connection>> m_connection;

    std::unique_ptr<RoomStream> roomStream = nullptr;

    explicit Private(std::optional<rust::Box<sdk::Connection>> connection)
        : m_connection(std::move(connection))
    {}

    ~Private() = default;
};

Connection::~Connection() = default;

Connection::Connection(std::optional<rust::Box<sdk::Connection>> rawConnection)
    : QObject(nullptr)
    , d(std::make_unique<Private>(std::move(rawConnection)))
{
}

QString Connection::matrixId() const
{
    return stringFromRust(connection()->matrix_id());
}

rust::Box<sdk::Connection> &Connection::connection() const
{
    return d->m_connection.value();
}

void Connection::open(const QString &roomId)
{
    Q_EMIT openRoom(room(roomId));
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
    connection()->logout();
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
    connection()->create_room(*options);
}

Room *Connection::room(const QString &id)
{
    //TODO cache room objects
    return new Room(connection()->room(stringToRust(id)));
}

bool Connection::hasRoom(const QString &id)
{
    return connection()->is_known_room(stringToRust(id));
}

Task *Connection::setDisplayName(const QString &displayName)
{
    const auto token = stringFromRust(connection()->set_display_name(stringToRust(displayName)));
    return new Task(token, this);
}

std::unique_ptr<RoomStream> Connection::roomStream()
{
    return std::make_unique<RoomStream>(this);
}
