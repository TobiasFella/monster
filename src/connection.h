// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QQmlEngine>
#include <qqmlintegration.h>

#include "lib.rs.h"

class Connection : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(bool loggedIn MEMBER m_loggedIn NOTIFY loggedInChanged)

public:
    Connection(QObject *parent = nullptr);
    ~Connection();

    Q_INVOKABLE void login(const QString &matrixId, const QString &password);
    Q_INVOKABLE void restore();

    rust::Box<sdk::Connection> &connection() const;
    QString matrixId() const;

    Q_INVOKABLE void open(const QString &roomId);

Q_SIGNALS:
    void loggedInChanged();
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void openRoom(const QString &roomId);

private:
    bool m_loggedIn = false;
    class Private;
    std::unique_ptr<Private> d;
};
