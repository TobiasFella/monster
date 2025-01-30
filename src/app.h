// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>
#include <QQmlEngine>

#include "lib.rs.h"

class App : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(bool loggedIn MEMBER m_loggedIn NOTIFY loggedInChanged)

public:
    static App *create(QQmlEngine *engine, QJSEngine *)
    {
        engine->setObjectOwnership(&instance(), QQmlEngine::CppOwnership);
        return &instance();
    }
    static App &instance() {
        static App _instance;
        return _instance;
    };
    ~App();

    Q_INVOKABLE void login(const QString &matrixId, const QString &password);

    rust::Box<sdk::Connection> &connection() const;

Q_SIGNALS:
    void connected();
    void loggedInChanged();

private:
    App();
    bool m_loggedIn = false;
    class Private;
    std::unique_ptr<Private> d;
};

