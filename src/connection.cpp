// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "connection.h"

#include <QDebug>
#include <QGuiApplication>

#include <qt6keychain/keychain.h>

#include "dispatcher.h"
#include "utils.h"

using namespace Qt::Literals::StringLiterals;

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

        const auto data = (*d->m_connection)->session();

        auto job = new QKeychain::WritePasswordJob(qAppName());
        job->setKey("0"_L1);
        job->setBinaryData({data.data(), (int)data.size()});
        job->setAutoDelete(true);
        job->start();
        Q_EMIT loggedInChanged();
    });
}

rust::Box<sdk::Connection> &Connection::connection() const
{
    return *d->m_connection;
}

void Connection::restore()
{
    auto job = new QKeychain::ReadPasswordJob(qAppName());
    job->setKey("0"_L1);
    job->setAutoDelete(true);
    job->start();
    connect(job, &QKeychain::Job::finished, this, [job, this]() {
        if (job->error() != QKeychain::NoError) {
            return;
        }
        const auto data = job->binaryData();
        d->m_connection = sdk::restore(rust::String(data.data(), data.size()));
    });

    connect(Dispatcher::instance(), &Dispatcher::connected, this, [this](const QString &) {
        m_loggedIn = true;
        d->matrixId = stringFromRust((*d->m_connection)->matrix_id());
        Q_EMIT loggedInChanged();
    });
}
