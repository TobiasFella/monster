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

/*!
 * \class PendingConnection
 * \brief A connection that is not yet ready to be used.
 * This is the initial state of a connection. Either login or registration has not finished yet, or the account is not loaded yet.
 * PendingConnection objects are acquired from Accounts and can be used to get the loaded connection after the ready() signal was emitted.
 */
class PendingConnection : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")
    Q_PROPERTY(QUrl oidcLoginUrl READ oidcLoginUrl NOTIFY oidcLoginUrlChanged)
    Q_PROPERTY(QString matrixId READ matrixId NOTIFY matrixIdChanged)

public:
    /**
     * \brief Get the connection for this PendingConnection.
     *
     * Only valid this after PendingConnection::ready() has been emitted.
     * \return the connection, or nullptr if called before that.
     */
    Q_INVOKABLE Quotient::Connection *connection();

    [[nodiscard]] QString matrixId() const;
    /*!
     * The url to open in a browser in order to continue login.
     * The client should open it automatically and/or present it for the user to open.
     * @return the url to open in a browser
     */
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
