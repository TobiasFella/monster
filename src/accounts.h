// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QList>
#include <QObject>
#include <QtQmlIntegration/qqmlintegration.h>

#include "pendingconnection.h"

#include "quotient_export.h"

namespace Quotient
{
class Connection;

class QUOTIENT_EXPORT Accounts : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QStringList availableAccounts READ availableAccounts NOTIFY availableAccountsChanged)

public:
    explicit Accounts(QObject *parent = nullptr);

    QStringList availableAccounts() const;

    //! Log in to an account that is not already logged in on the client
    Q_INVOKABLE Quotient::PendingConnection *loginWithPassword(const QString &matrixId, const QString &password);

    //! Load an account that is already logged in (i.e., which is listed in Accounts::availableAccounts)
    Q_INVOKABLE Quotient::PendingConnection *loadAccount(const QString &matrixId);

    void newConnection(Quotient::Connection *connection);

Q_SIGNALS:
    void availableAccountsChanged();

private:
    QStringList m_availableAccounts;
    QStringList m_allAccounts;

    void loadAccounts();
    void saveAccounts();
};

}
