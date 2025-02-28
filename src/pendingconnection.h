// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QtQmlIntegration/qqmlintegration.h>

#include "connection.h"

#include "quotient_export.h"

namespace Quotient
{
class Accounts;

class QUOTIENT_EXPORT PendingConnection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

public:
    Q_INVOKABLE static Quotient::PendingConnection *loginWithPassword(const QString &matrixId, const QString &password, Accounts *accounts);
    Q_INVOKABLE static Quotient::PendingConnection *loadAccount(const QString &matrixId, Accounts *accounts);

    /**
     * @brief Get the connection for this PendingConnection.
     *
     * Only call this after PendingConnection::ready() has been emitted.
     * Only call this once. The PendingConnection object is no longer in a valid state after.
     */
    Q_INVOKABLE Quotient::Connection *connection();

    QString matrixId() const;
    ~PendingConnection();

Q_SIGNALS:
    void ready();

private:
    class Private;
    std::unique_ptr<Private> d;
    PendingConnection();
    void setMatrixId(const QString &matrixId);

    // TODO: Make this an error enum instead
    bool m_ready = false;
    QString m_matrixId;
    Quotient::Accounts *m_accounts;
};

}
