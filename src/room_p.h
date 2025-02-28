// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include "lib.rs.h"

#include "room.h"

using namespace Quotient;

struct RustRoomWrapper {
    std::optional<rust::Box<sdk::Room>> room;
};

class Room::Private
{
public:
    ~Private()
    {
        delete wrapper;
    }
    RustRoomWrapper *wrapper = nullptr;
};
