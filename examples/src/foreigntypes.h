// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include <QQmlEngine>

#include <QuotientNg/Accounts>
#include <QuotientNg/PendingConnection>

#include "roomavatarimageprovider.h"
#include "roomsmodel.h"
#include "timelinemodel.h"

struct ForeignAccountRegistry {
    Q_GADGET
    QML_FOREIGN(Quotient::Accounts)
    QML_NAMED_ELEMENT(Accounts)
};

struct ForeignPendingConnection {
    Q_GADGET
    QML_FOREIGN(Quotient::PendingConnection)
    QML_NAMED_ELEMENT(PendingConnection)
    QML_UNCREATABLE("")
};

struct ForeignRoomsModel {
    Q_GADGET
    QML_FOREIGN(RoomsModel)
    QML_NAMED_ELEMENT(RoomsModel)
};

struct ForeignTimelineModel {
    Q_GADGET
    QML_FOREIGN(TimelineModel)
    QML_NAMED_ELEMENT(TimelineModel)
};

struct ForeignConnection {
    Q_GADGET
    QML_FOREIGN(Quotient::Connection)
    QML_NAMED_ELEMENT(Connection)
    QML_UNCREATABLE("")
};

struct ForeignRoomAvatarImageProvider {
    Q_GADGET
    QML_FOREIGN(RoomAvatarImageProvider)
    QML_NAMED_ELEMENT(RoomAvatarImageProvider)
    QML_SINGLETON

    static RoomAvatarImageProvider *create(QQmlEngine *engine, QJSEngine *)
    {
        return RoomAvatarImageProvider::create(engine, engine);
    }
};
