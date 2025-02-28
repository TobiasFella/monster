// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "accounts.h"

#include <QCoreApplication>
#include <QDebug>
#include <QDir>
#include <QStandardPaths>

#include <qt6keychain/keychain.h>

#include "pendingconnection.h"

using namespace Qt::Literals::StringLiterals;
using namespace Quotient;

Accounts::Accounts(QObject *parent)
    : QObject(parent)
{
    QMetaObject::invokeMethod(this, &Accounts::loadAccounts);
}

void Accounts::loadAccounts()
{
    const auto dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    QDir().mkpath(dir);
    auto file = dir + QDir::separator() + u"Accounts"_s;
    QFile accounts(file);
    accounts.open(QIODevice::ReadWrite);
    auto data = QString::fromUtf8(accounts.readAll()).split(u'\n');

    for (const auto &account : data) {
        if (!account.isEmpty()) {
            m_allAccounts += account;
            m_availableAccounts += account;
        }
    }
    Q_EMIT availableAccountsChanged();
}

QStringList Accounts::availableAccounts() const
{
    return m_availableAccounts;
}

Quotient::PendingConnection *Accounts::loginWithPassword(const QString &matrixId, const QString &password)
{
    auto pending = PendingConnection::loginWithPassword(matrixId, password, this);

    m_allAccounts += pending->matrixId();

    saveAccounts();

    return pending;
}

Quotient::PendingConnection *Accounts::loadAccount(const QString &matrixId)
{
    return PendingConnection::loadAccount(matrixId, this);
}

void Accounts::newConnection(Connection *connection)
{
    connect(connection, &Connection::loggedOut, this, [connection, this]() {
        m_allAccounts.removeAll(connection->matrixId());
        saveAccounts();
        auto job = new QKeychain::DeletePasswordJob(qAppName());
        job->setKey(connection->matrixId());
        job->setAutoDelete(true);
        job->start();
        connect(job, &QKeychain::Job::finished, this, [job]() {
            if (job->error() != QKeychain::NoError) {
                qWarning() << "Failed to delete from keychain" << job->error();
            }
        });
    });
}

void Accounts::saveAccounts()
{
    const auto dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    auto file = dir + QDir::separator() + u"Accounts"_s;
    QFile accounts(file);
    accounts.open(QIODevice::WriteOnly);
    accounts.write(m_allAccounts.join(u'\n').toUtf8());
    accounts.close();
}
