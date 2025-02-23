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

QString Room::id() const
{
    return stringFromRust((*d->wrapper->room)->id());
}

rust::u8 Room::state() const
{
    return (*d->wrapper->room)->state();
}

bool Room::isSpace() const
{
    return (*d->wrapper->room)->is_space();
}

QString Room::roomType() const
{
    return stringFromRust((*d->wrapper->room)->room_type());
}

QString Room::displayName() const
{
    return stringFromRust((*d->wrapper->room)->display_name());
}

bool Room::isTombstoned() const
{
    return (*d->wrapper->room)->is_tombstoned();
}

rust::Box<sdk::RoomTombstoneEventContent> Room::tombstone() const
{
    return (*d->wrapper->room)->tombstone();
}

QString Room::topic() const
{
    return stringFromRust((*d->wrapper->room)->topic());
}

rust::u64 Room::numUnreadMessages() const
{
    return (*d->wrapper->room)->num_unread_messages();
}

rust::u64 Room::numUnreadMentions() const
{
    return (*d->wrapper->room)->num_unread_mentions();
}

bool Room::isFavourite() const
{
    return (*d->wrapper->room)->is_favourite();
}

bool Room::isLowPriority() const
{
    return (*d->wrapper->room)->is_low_priority();
}

Room::~Room() = default;

Room::Room(rust::Box<sdk::Room> room, QObject *parent)
    : QObject(parent)
    , d(std::make_unique<Private>(new RustRoomWrapper(std::move(room))))
{
}
