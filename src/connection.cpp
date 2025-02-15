// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "connection.h"

#include <QDebug>
#include <QGuiApplication>

#include <qt6keychain/keychain.h>

#include "lib.rs.h"
#include "utils.h"
#include "dispatcher.h"

using namespace Qt::Literals::StringLiterals;
using namespace Quotient;

class Connection::Private
{
public:
    RustConnectionWrapper *wrapper = nullptr;

    Private(RustConnectionWrapper *wrapper)
        : wrapper(wrapper)
    {}

    ~Private() {
        delete wrapper;
    }
};

Connection::~Connection() = default;

Connection::Connection(RustConnectionWrapper *wrapper)
    : QObject(nullptr)
    , d(std::make_unique<Private>(wrapper))
{
}

QString Connection::matrixId() const
{
    return stringFromRust(connection()->matrix_id());
}

rust::Box<sdk::Connection> &Connection::connection() const
{
    return *d->wrapper->m_connection;
}

void Connection::open(const QString &roomId)
{
    Q_EMIT openRoom(roomId);
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
