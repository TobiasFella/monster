// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>

#include "connection.h"

enum class ConnectionType;

namespace Quotient
{
class Accounts;

class PendingConnection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")
    Q_PROPERTY(QUrl oidcLoginUrl READ oidcLoginUrl NOTIFY oidcLoginUrlChanged)
    Q_PROPERTY(QString matrixId READ matrixId NOTIFY matrixIdChanged)

public:
    /**
     * @brief Get the connection for this PendingConnection.
     *
     * Only call this after PendingConnection::ready() has been emitted.
     * Only call this once. The PendingConnection object is no longer in a valid state after.
     */
    Q_INVOKABLE Quotient::Connection *connection();

    [[nodiscard]] QString matrixId() const;
    [[nodiscard]] QUrl oidcLoginUrl() const;
    ~PendingConnection() override;

Q_SIGNALS:
    void matrixIdChanged();
    void ready();
    void oidcLoginUrlChanged();

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
    QUrl m_oidcLoginUrl;
    std::optional<rust::Box<sdk::Connection>> m_rawConnection;
    Accounts *m_accounts = nullptr;
    Connection *m_connection = nullptr;
    void initialize(ConnectionType type);
    void setReady(bool ready);
};

}
