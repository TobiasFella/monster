// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>
#include <QQmlEngine>

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

Q_SIGNALS:
    void loggedInChanged();
    void avatarLoaded(const QString &roomId, const QByteArray &data);

private:
    bool m_loggedIn = false;
    class Private;
    std::unique_ptr<Private> d;
};

