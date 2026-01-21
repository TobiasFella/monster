// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "pendingconnection.h"

#include <QCoreApplication>

#include <QDesktopServices>
#include <QUrl>
#include <qt6keychain/keychain.h>

#include "accounts.h"
#include "connection.h"
#include "dispatcher.h"
#include "utils.h"

using namespace Quotient;

PendingConnection::~PendingConnection() {
}

PendingConnection::PendingConnection() = default;

void PendingConnection::setMatrixId(const QString &matrixId)
{
    m_matrixId = matrixId;
}

Quotient::PendingConnection *PendingConnection::loginWithPassword(const QString &matrixId, const QString &password, Accounts *accounts)
{
    auto pendingConnection = new PendingConnection();
    pendingConnection->setMatrixId(matrixId);
    pendingConnection->wrapper = new RustConnectionWrapper { sdk::init(stringToRust(matrixId), stringToRust(password)) };
    // TODO: Disconnect this once logged in
    connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection](const QString &matrixId) {
        if (matrixId != pendingConnection->m_matrixId) {
            return;
        }

        const auto data = (*pendingConnection->wrapper->m_connection)->session();

        auto job = new QKeychain::WritePasswordJob(qAppName());
        job->setKey(matrixId);
        job->setBinaryData({data.data(), (int)data.size()});
        job->setAutoDelete(true);
        job->start();

        connect(job, &QKeychain::WritePasswordJob::finished, pendingConnection, [pendingConnection](const auto &job) {
            if (job->error() != QKeychain::NoError) {
                qWarning() << "Failed to write to keychain" << job->error();
                //TODO error the entire pendingConnection;
                return;
            }
            pendingConnection->m_ready = true;
            Q_EMIT pendingConnection->ready();
        });
    });
    pendingConnection->m_accounts = accounts;
    return pendingConnection;
}

PendingConnection *PendingConnection::loadAccount(const QString &matrixId, Accounts *accounts)
{
    auto pendingConnection = new PendingConnection();
    pendingConnection->setMatrixId(matrixId);

    auto job = new QKeychain::ReadPasswordJob(qAppName());
    job->setKey(matrixId);
    job->setAutoDelete(true);
    job->start();
    connect(job, &QKeychain::Job::finished, pendingConnection, [job, pendingConnection]() {
        if (job->error() != QKeychain::NoError) {
            //TODO error entirely here
            qWarning() << "Failed to read from keychain" << job->error();
            return;
        }
        const auto data = job->binaryData();
        pendingConnection->wrapper = new RustConnectionWrapper { sdk::restore(rust::String(data.data(), data.size())) };
        connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection](const QString &) {
            pendingConnection->m_ready = true;
            Q_EMIT pendingConnection->ready();
        });
    });
    pendingConnection->m_accounts = accounts;
    return pendingConnection;
}

PendingConnection *PendingConnection::loginWithOidc(const QString &serverName, Accounts *accounts)
{
    const auto pendingConnection = new PendingConnection();
    pendingConnection->wrapper = new RustConnectionWrapper { sdk::init_oidc(stringToRust(serverName)) };
    //TODO connectuntil
    connect(Dispatcher::instance(), &Dispatcher::oidcLoginUrlAvailable, pendingConnection, [pendingConnection, serverName](const auto &server, const auto &url) {
        if (server != serverName) {
            return;
        }
        pendingConnection->m_oidcLoginUrl = QUrl(url);
        Q_EMIT pendingConnection->oidcLoginUrlChanged();
        QDesktopServices::openUrl(pendingConnection->m_oidcLoginUrl);
    });
    //TODO: Deduplicate
    connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection, serverName](const QString &matrixId) {
        // NOTE: this is not a matrix id, but for simplicity, we just use the same function
        if (matrixId != serverName) {
            return;
        }

        const auto data = (*pendingConnection->wrapper->m_connection)->session();

        const auto job = new QKeychain::WritePasswordJob(qAppName());
        job->setKey(matrixId);
        job->setBinaryData({data.data(), (int)data.size()});
        job->setAutoDelete(true);
        job->start();

        connect(job, &QKeychain::WritePasswordJob::finished, pendingConnection, [pendingConnection](const auto &job) {
            if (job->error() != QKeychain::NoError) {
                qWarning() << "Failed to write to keychain" << job->error();
                //TODO error the entire pendingConnection;
                return;
            }
            pendingConnection->m_ready = true;
            Q_EMIT pendingConnection->ready();
        });
    });
    pendingConnection->m_accounts = accounts;
    return pendingConnection;
}

Connection *PendingConnection::connection()
{
    if (!m_ready) {
        return {};
    }
    auto connection = new Connection(wrapper);
    m_accounts->newConnection(connection);
    wrapper = nullptr;
    return connection;
}

QString PendingConnection::matrixId() const
{
    return m_matrixId;
}

QUrl PendingConnection::oidcLoginUrl() const
{
    return m_oidcLoginUrl;
}
