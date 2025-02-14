// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "accounts.h"

#include <QStandardPaths>
#include <QDebug>
#include <QDir>

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
    auto pending = PendingConnection::loginWithPassword(matrixId, password);

    m_allAccounts += pending->matrixId();

    const auto dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    auto file = dir + QDir::separator() + u"Accounts"_s;
    QFile accounts(file);
    accounts.open(QIODevice::ReadWrite);
    accounts.write(m_allAccounts.join(u'\n').toUtf8());
    accounts.close();
    //TODO: Save to file

    return pending;
}

Quotient::PendingConnection *Accounts::loadAccount(const QString &matrixId)
{
    return PendingConnection::loadAccount(matrixId);
}
