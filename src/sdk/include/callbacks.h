// SPDX-FileCopyrightText: Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later


#pragma once

#include <cstdint>
#include "rust/cxx.h"

namespace sdk {
class RoomListRoom;
}

void shim_connected();
void shim_rooms_changed(std::uint8_t op, std::size_t index, std::size_t length);
void shim_timeline_changed(std::uint8_t op, std::size_t index, std::size_t length);
void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> data);
