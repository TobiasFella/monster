// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>

#include "pendingconnection.h"

namespace Quotient
{
class Connection;

class Accounts : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QStringList availableAccounts READ availableAccounts NOTIFY availableAccountsChanged)

public:
    explicit Accounts(QObject *parent = nullptr);

    [[nodiscard]] QStringList availableAccounts() const;

    //! Log in to an account that is not already logged in on the client, with native matrix login
    Q_INVOKABLE Quotient::PendingConnection *loginWithPassword(const QString &matrixId, const QString &password);

    //! Log in to an account that is not already logged in on the client, with oidc login
    Q_INVOKABLE Quotient::PendingConnection *loginWithOidc(const QString &serverName);

    //! Load an account that is already logged in (i.e., which is listed in Accounts::availableAccounts)
    Q_INVOKABLE Quotient::PendingConnection *loadAccount(const QString &matrixId);

Q_SIGNALS:
    void availableAccountsChanged();

private:
    friend class PendingConnection;
    QStringList m_availableAccounts;
    QList<PendingConnection *> m_loadedAccounts;


    void accountLoaded(PendingConnection *connection);
    void accountLoggedOut(const QString &matrixId);
    void loadAccounts();
    void saveAccounts() const;
};

}
