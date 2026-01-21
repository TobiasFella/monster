// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>

#include "connection.h"

namespace Quotient
{
class Accounts;

class PendingConnection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

public:
    /**
     * @brief Get the connection for this PendingConnection.
     *
     * Only call this after PendingConnection::ready() has been emitted.
     * Only call this once. The PendingConnection object is no longer in a valid state after.
     */
    Q_INVOKABLE Quotient::Connection *connection();

    QString matrixId() const;
    ~PendingConnection() override;

Q_SIGNALS:
    void ready();

private:
    friend class Accounts;
    PendingConnection();
    void setMatrixId(const QString &matrixId);

    static Quotient::PendingConnection *loginWithPassword(const QString &matrixId, const QString &password, Accounts *accounts);
    static Quotient::PendingConnection *loadAccount(const QString &matrixId, Accounts *accounts);
    static Quotient::PendingConnection *loginWithOidc(const QString &serverName, Accounts *accounts);

    //TODO: Make this an error enum instead
    bool m_ready = false;
    QString m_matrixId;
    RustConnectionWrapper *wrapper = nullptr;
    Quotient::Accounts *m_accounts;
};

}
