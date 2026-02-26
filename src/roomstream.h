// SPDX-FileCopyrightText: 2025 James Graham <james.h.graham@protonmail.com>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>

#include "ffi.rs.h"

namespace Quotient {
    class Connection;

/*
 * Provides a stream of room list updates for a given connection.
 *
 * Designed to hide dispatcher and make sure that other components don't have to know much
 * about the guts of the matrix_rust_sdk bindings. A user just see a list of updates to
 * their room list.
 */
class RoomStream : public QObject
{
    Q_OBJECT

public:
    RoomStream(Quotient::Connection *connection);
    ~RoomStream();

    bool startStream();
    bool running();

    rust::Box<sdk::RoomListVecDiff> next();

Q_SIGNALS:
    void roomsUpdate();

private:
    class Private;
    std::unique_ptr<Private> d;
};
}
