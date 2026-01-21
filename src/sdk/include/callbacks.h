// SPDX-FileCopyrightText: Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later


#pragma once

#include <cstdint>
#include "rust/cxx.h"

namespace sdk {
class RoomListItem;
}

void shim_connected(rust::String matrixId);
void shim_rooms_changed(rust::String matrixId);
void shim_timeline_changed(rust::String matrixId, rust::String roomId);
void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> data);
void shim_logged_out(rust::String matrixId);
