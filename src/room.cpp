// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "room.h"

#include "room_p.h"
#include "utils.h"

using namespace Quotient;

QString Room::displayName() const
{
    return stringFromRust((*d->wrapper->room)->display_name());
}

QString Room::id() const
{
    return stringFromRust((*d->wrapper->room)->id());
}

Room::~Room() = default;

Room::Room(std::unique_ptr<Private> d, QObject *parent)
    : QObject(parent)
    , d(std::move(d))
{
}
