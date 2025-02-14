// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "pendingconnection.h"

#include <QCoreApplication>

#include <qt6keychain/keychain.h>

#include "connection.h"
#include "dispatcher.h"
#include "utils.h"


using namespace Quotient;

PendingConnection::~PendingConnection() {
    qWarning() << ".onf";
}

PendingConnection::PendingConnection() = default;

void PendingConnection::setMatrixId(const QString &matrixId)
{
    m_matrixId = matrixId;
}

Quotient::PendingConnection *PendingConnection::loginWithPassword(const QString &matrixId, const QString &password)
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
    return pendingConnection;
}

Quotient::PendingConnection *PendingConnection::loadAccount(const QString &matrixId)
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
    return pendingConnection;
}

Quotient::Connection *PendingConnection::connection()
{
    if (!m_ready) {
        return {};
    }
    auto connection = new Connection(wrapper);
    wrapper = nullptr;
    return connection;
}

QString PendingConnection::matrixId() const
{
    return m_matrixId;
}
