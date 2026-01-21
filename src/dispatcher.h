// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>

/* Dispatcher is used internally to redistribute "callbacks" from Rust to C++.
 * It's the only nice way i have come up with for passing the result of async
 * functions back to C++. It would be nice to have something closer to Qt's
 * connections, where a callback is called on a specific receiver object. But I
 * think Cxx is too limited for that
 */
class Dispatcher : public QObject
{
    Q_OBJECT

public:
    static Dispatcher *instance()
    {
        static Dispatcher _instance;
        return &_instance;
    }

Q_SIGNALS:
    void connected(const QString &matrixId);
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void roomsUpdate(const QString &matrixId);
    void timelineUpdate(const QString &matrix_id, const QString &room_id);
    void loggedOut(const QString &matrixId);
    void oidcLoginUrlAvailable(const QString &serverName, const QString &url);

private:
    Dispatcher();
};
