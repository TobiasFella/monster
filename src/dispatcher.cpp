// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "dispatcher.h"

#include "utils.h"

void shim_connected(rust::String userId)
{
    Q_EMIT Dispatcher::instance()->connected(stringFromRust(userId));
}

void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> _data)
{
    Q_EMIT Dispatcher::instance()->avatarLoaded(stringFromRust(roomId), QByteArray((const char *)_data.data(), _data.size()));
}

void shim_rooms_changed(rust::String matrixId, std::uint8_t op, std::size_t from, std::size_t to)
{
    Q_EMIT Dispatcher::instance()->roomsUpdate(stringFromRust(matrixId), op, from, to);
}

void shim_timeline_changed(rust::String matrix_id, rust::String room_id, std::uint8_t op, std::size_t from, std::size_t to)
{
    Q_EMIT Dispatcher::instance()->timelineUpdate(stringFromRust(matrix_id), stringFromRust(room_id), op, from, to);
}

Dispatcher::Dispatcher()
    : QObject(nullptr)
{
}
