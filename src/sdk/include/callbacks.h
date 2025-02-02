// SPDX-FileCopyrightText: Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later


#pragma once

#include <cstdint>
#include "rust/cxx.h"

namespace sdk {
class RoomListRoom;
}

void shim_connected(rust::String matrixId);
void shim_rooms_changed(rust::String matrixId, std::uint8_t op, std::size_t from, std::size_t to);
void shim_timeline_changed(rust::String matrixId, rust::String roomId, std::uint8_t op, std::size_t index, std::size_t length);
void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> data);
