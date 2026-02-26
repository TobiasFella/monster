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

enum class ConnectionType
{
    New,
    Existing,
};

PendingConnection::~PendingConnection() = default;

PendingConnection::PendingConnection() = default;

void PendingConnection::setMatrixId(const QString &matrixId)
{
    m_matrixId = matrixId;
    Q_EMIT matrixIdChanged();
}

PendingConnection *PendingConnection::loginWithPassword(const QString &matrixId, const QString &password, Accounts *accounts)
{
    auto pendingConnection = new PendingConnection();
    pendingConnection->setMatrixId(matrixId);
    pendingConnection->m_rawConnection = sdk::init(stringToRust(matrixId), stringToRust(password));
    // TODO: Disconnect this once logged in
    connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection](const QString &matrixId) {
        if (matrixId != pendingConnection->matrixId()) {
            return;
        }
        pendingConnection->initialize(ConnectionType::New);
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
    connect(job, &QKeychain::Job::finished, pendingConnection, [job, pendingConnection] {
        if (job->error() != QKeychain::NoError) {
            //TODO error entirely here
            qWarning() << "Failed to read from keychain" << job->error();
            return;
        }
        const auto data = job->binaryData();
        pendingConnection->m_rawConnection = sdk::restore(rust::String(data.data(), data.size()));
        connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection](const QString &) {
            pendingConnection->initialize(ConnectionType::Existing);
        });
    });
    pendingConnection->m_accounts = accounts;
    return pendingConnection;
}

PendingConnection *PendingConnection::loginWithOidc(const QString &serverName, Accounts *accounts)
{
    const auto pendingConnection = new PendingConnection();
    pendingConnection->m_rawConnection = sdk::init_oidc(stringToRust(serverName));
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
    connect(Dispatcher::instance(), &Dispatcher::connected, pendingConnection, [pendingConnection, serverName](const QString &server) {
        if (server != serverName) {
            return;
        }

        pendingConnection->setMatrixId(stringFromRust((*pendingConnection->m_rawConnection)->matrix_id()));
        pendingConnection->initialize(ConnectionType::New);
    });
    pendingConnection->m_accounts = accounts;
    return pendingConnection;
}

Connection *PendingConnection::connection()
{
    if (!m_ready) {
        return {};
    }

    if (!m_connection) {
        m_connection = new Connection(std::move(m_rawConnection));
        m_rawConnection = std::nullopt;
    }

    return m_connection;
}

QString PendingConnection::matrixId() const
{
    return m_matrixId;
}

QUrl PendingConnection::oidcLoginUrl() const
{
    return m_oidcLoginUrl;
}

void PendingConnection::initialize(ConnectionType type)
{
    m_accounts->accountLoaded(this);

    connect(connection(), &Connection::loggedOut, connection(), [this] {
        m_accounts->accountLoggedOut(matrixId());
        auto job = new QKeychain::DeletePasswordJob(qAppName());
        job->setKey(matrixId());
        job->setAutoDelete(true);
        job->start();
        connect(job, &QKeychain::Job::finished, this, [job] {
            if (job->error() != QKeychain::NoError) {
                qWarning() << "Failed to delete key" << job->error();
            }
        });
    });

    if (type == ConnectionType::New) {
        const auto job = new QKeychain::WritePasswordJob(qAppName());
        job->setKey(matrixId());
        job->setBinaryData(bytesFromRust((*m_rawConnection)->session()));
        job->setAutoDelete(true);
        job->start();

        connect(job, &QKeychain::WritePasswordJob::finished, this, [this](const auto &job) {
            if (job->error() != QKeychain::NoError) {
                qWarning() << "Failed to write to keychain" << job->error();
                //TODO error the entire pendingConnection;
                return;
            }
            setReady(true);
        });
    } else {
        setReady(true);
    }
}
void PendingConnection::setReady(const bool ready)
{
    if (m_ready == ready) {
        return;
    }
    m_ready = ready;
    Q_EMIT this->ready();
}