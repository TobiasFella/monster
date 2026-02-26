// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "dispatcher.h"

#include "sdk/include/callbacks.h"
#include "utils.h"

void shim_connected(rust::String userId)
{
    Q_EMIT Dispatcher::instance()->connected(stringFromRust(userId));
}

void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> _data)
{
    Q_EMIT Dispatcher::instance()->avatarLoaded(stringFromRust(roomId), QByteArray((const char *)_data.data(), _data.size()));
}

void shim_rooms_changed(rust::String matrixId)
{
    Q_EMIT Dispatcher::instance()->roomsUpdate(stringFromRust(matrixId));
}

void shim_timeline_changed(rust::String matrix_id, rust::String room_id)
{
    Q_EMIT Dispatcher::instance()->timelineUpdate(stringFromRust(matrix_id), stringFromRust(room_id));
}

void shim_logged_out(rust::String matrixId)
{
    Q_EMIT Dispatcher::instance()->loggedOut(stringFromRust(matrixId));
}

void shim_oidc_login_url_available(rust::String serverName, rust::String url)
{
    Q_EMIT Dispatcher::instance()->oidcLoginUrlAvailable(stringFromRust(serverName), stringFromRust(url));
}

void task_done(rust::String token)
{
    Q_EMIT Dispatcher::instance()->taskDone(stringFromRust(token));
}

Dispatcher::Dispatcher()
    : QObject(nullptr)
{
}
