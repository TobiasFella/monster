// SPDX-FileCopyrightText: 2025 James Graham <james.h.graham@protonmail.com>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomstream.h"

#include "connection.h"
#include "dispatcher.h"
#include "connection_p.h"

using namespace Quotient;

class RoomStream::Private
{
public:
    QString matrixId;
    std::optional<rust::Box<sdk::Rooms>> rooms;

    void roomsUpdate();

    RoomStream* q = nullptr;
};

RoomStream::RoomStream(Quotient::Connection *connection)
    : QObject(connection)
    , d(std::make_unique<Private>())
{
    Q_ASSERT(connection);

    // See "Accessing the Public Class" section in
    // https://marcmutz.wordpress.com/translated-articles/pimp-my-pimpl-%E2%80%94-reloaded/
    d->q = this;

    d->matrixId = connection->matrixId();

    connect(Dispatcher::instance(), &Dispatcher::roomsUpdate, this, [this](const auto &matrixId) {
        if (matrixId != d->matrixId) {
            return;
        }
        d->roomsUpdate();
    }, Qt::QueuedConnection);
}

RoomStream::~RoomStream() = default;

bool RoomStream::startStream()
{
    const auto connection = dynamic_cast<Quotient::Connection *>(parent());
    if (connection == nullptr) {
        return false;
    }
    connection->d->connection()->slide();
    return true;
}

bool RoomStream::running()
{
    return d->rooms != std::nullopt;
}

rust::Box<sdk::RoomListVecDiff> RoomStream::next()
{
    return (*d->rooms)->queue_next();
}

void RoomStream::Private::roomsUpdate()
{
    QMetaObject::invokeMethod(
        q,
        [this]() {
            while ((*rooms)->has_queued_item()) {
                q->roomsUpdate();
            }
        },
        Qt::QueuedConnection);
}
