// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "connection.h"

#include <QDebug>

#include "utils.h"
#include "dispatcher.h"

class Connection::Private
{
public:
    std::optional<rust::Box<sdk::Connection>> m_connection;
    QString matrixId;
};

Connection::~Connection() = default;

Connection::Connection(QObject *parent)
    : QObject(parent)
    , d(std::make_unique<Private>())
{
}

QString Connection::matrixId() const
{
    return d->matrixId;
}

void Connection::login(const QString &matrixId, const QString &password)
{
    d->m_connection = sdk::init(stringToRust(matrixId), stringToRust(password));
    d->matrixId = matrixId;

    connect(Dispatcher::instance(), &Dispatcher::connected, this, [this](const QString &userId) {
        if (userId != d->matrixId) {
            return;
        }
        m_loggedIn = true;
        Q_EMIT loggedInChanged();
    });
}

rust::Box<sdk::Connection> &Connection::connection() const
{
    return *d->m_connection;
}


