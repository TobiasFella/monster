// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>

class Dispatcher : public QObject
{
    Q_OBJECT

public:
    static Dispatcher *instance() {
        static Dispatcher _instance;
        return &_instance;
    }

Q_SIGNALS:
    void connected(const QString &userId);
    void avatarLoaded(const QString &roomId, const QByteArray &data);
    void roomsUpdate(const QString &matrixId, std::uint8_t op, std::size_t from, std::size_t to);
    void timelineUpdate(const QString &matrix_id, const QString &room_id, std::uint8_t op, std::size_t from, std::size_t to);

private:
    Dispatcher();
};
