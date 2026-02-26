// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "accounts.h"

#include <QStandardPaths>
#include <QDebug>
#include <QDir>
#include <QGuiApplication>

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
    (void) QDir().mkpath(dir);
    const auto file = dir + QDir::separator() + u"Accounts"_s;
    QFile accounts(file);
    (void) accounts.open(QIODevice::ReadWrite);
    const auto data = QString::fromUtf8(accounts.readAll()).split(u'\n');

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
    return QList(m_availableAccounts.begin(), m_availableAccounts.end());
}

PendingConnection *Accounts::loginWithPassword(const QString &matrixId, const QString &password)
{
    auto pending = PendingConnection::loginWithPassword(matrixId, password, this);

    m_allAccounts += pending->matrixId();

    saveAccounts();

    return pending;
}

PendingConnection *Accounts::loadAccount(const QString &matrixId)
{
    return PendingConnection::loadAccount(matrixId, this);
}

void Accounts::newConnection(Connection *connection)
{
    connect(connection, &Connection::loggedOut, this, [connection, this](){
        m_allAccounts.remove(connection->matrixId());
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
    accounts.write(QList(m_allAccounts.begin(), m_allAccounts.end()).join(u'\n').toUtf8());
    accounts.close();
}

PendingConnection *Accounts::loginWithOidc(const QString &serverName)
{
    auto pending = PendingConnection::loginWithOidc(serverName, this);
    connect(pending, &PendingConnection::ready, this, [this, pending] {
        m_allAccounts += pending->matrixId();
        saveAccounts();
    });
    return pending;
}
