// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "accounts.h"

#include <QStandardPaths>
#include <QDebug>
#include <QDir>
#include <QGuiApplication>

#include <qt6keychain/keychain.h>

#include "pendingconnection.h"

using namespace Qt::StringLiterals;
using namespace Quotient;

Accounts::Accounts(QObject *parent)
    : QObject(parent)
{
    QMetaObject::invokeMethod(this, &Accounts::loadAccounts);
}

void Accounts::accountLoaded(PendingConnection *connection)
{
    if (!m_availableAccounts.contains(connection->matrixId())) {
        m_availableAccounts.append(connection->matrixId());
        Q_EMIT availableAccountsChanged();
    }
    m_loadedAccounts.append(connection);
    saveAccounts();
}

void Accounts::accountLoggedOut(const QString &matrixId)
{
    m_availableAccounts.removeAll(matrixId);
    Q_EMIT availableAccountsChanged();
    //TODO remove account from loaded accounts;
    saveAccounts();
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
        if (!account.isEmpty() && !m_availableAccounts.contains(account)) {
            m_availableAccounts += account;
        }
    }
    Q_EMIT availableAccountsChanged();
}

QStringList Accounts::availableAccounts() const
{
    return m_availableAccounts;
}

PendingConnection *Accounts::loginWithPassword(const QString &matrixId, const QString &password)
{
    return PendingConnection::loginWithPassword(matrixId, password, this);
}

PendingConnection *Accounts::loadAccount(const QString &matrixId)
{
    return PendingConnection::loadAccount(matrixId, this);
}

void Accounts::saveAccounts() const
{
    const auto dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    const auto file = dir + QDir::separator() + u"Accounts"_s;
    QFile accounts(file);
    if (!accounts.open(QIODevice::WriteOnly)) {
        qCritical() << Q_FUNC_INFO << "Error opening accounts file";
    }
    accounts.write(m_availableAccounts.join(u'\n').toUtf8());
    accounts.close();
}

PendingConnection *Accounts::loginWithOidc(const QString &serverName)
{
    return PendingConnection::loginWithOidc(serverName, this);
}
