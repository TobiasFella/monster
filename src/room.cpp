// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "room.h"

#include "utils.h"

using namespace Quotient;

struct RustRoomWrapper
{
    std::optional<rust::Box<sdk::Room>> room;
};

class Room::Private
{
public:
    ~Private() {
        delete wrapper;
    }
    RustRoomWrapper *wrapper = nullptr;
};

QString Room::displayName() const
{
    return stringFromRust((*d->wrapper->room)->display_name());
}

QString Room::id() const
{
    return stringFromRust((*d->wrapper->room)->id());
}

Room::~Room() = default;

Room::Room(rust::Box<sdk::Room> room, QObject *parent)
    : QObject(parent)
    , d(std::make_unique<Private>(new RustRoomWrapper(std::move(room))))
{
}
