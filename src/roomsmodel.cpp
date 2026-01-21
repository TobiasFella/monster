// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomsmodel.h"

#include "connection.h"
#include "lib.rs.h"
#include "roomstream.h"
#include "utils.h"

#include <QPointer>

using namespace Quotient;

struct RoomWrapper
{
    std::optional<rust::Box<sdk::RoomListItem>> item;
};

class RoomsModel::Private
{
public:
    QPointer<Quotient::Connection> connection;
    std::unique_ptr<RoomStream> roomStream = nullptr;
    QList<RoomWrapper *> items;

    void roomsUpdate();

    RoomsModel* q = nullptr;

    ~Private() {
        for (const auto &item : items) {
            delete item;
        }
    }
};

RoomsModel::~RoomsModel() = default;

RoomsModel::RoomsModel(QObject *parent)
    : QAbstractListModel(parent)
    , d(std::make_unique<Private>())
{
    d->q = this;
}

QHash<int, QByteArray> RoomsModel::roleNames() const
{
    return {
        {RoomsModel::IdRole, "roomId"},
        {RoomsModel::DisplayNameRole, "displayName"},
        {RoomsModel::AvatarUrlRole, "avatarUrl"},
        {RoomsModel::RoomRole, "room"},
    };
}

QVariant RoomsModel::data(const QModelIndex &index, int role) const
{
    Q_UNUSED(role);
    const auto row = index.row();

    if (role == IdRole) {
        return stringFromRust((*d->items[row]->item)->id()).toHtmlEscaped();
    } else if (role == DisplayNameRole) {
        return stringFromRust((*d->items[row]->item)->display_name()).toHtmlEscaped();
    } else if (role == AvatarUrlRole) {
        return QStringLiteral("image://roomavatar/%1").arg(stringFromRust((*d->items[row]->item)->id()));
    } else if (role == RoomRole) {
        return QVariant::fromValue(d->connection->room(stringFromRust((*d->items[row]->item)->id())));
    }
    return {};
}

int RoomsModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) {
        return {};
    }
    return d->items.size();
}

void RoomsModel::Private::roomsUpdate()
{
    const auto diff = roomStream->next();

    switch (diff->op()) {
        case 0: { // Append
            auto newItems = diff->items_vec();
            q->beginInsertRows({}, q->rowCount({}), q->rowCount({}) + newItems.size());
            for (const auto &it : newItems) {
                auto timelineItem = new RoomWrapper{it.box_me()};
                items.append(timelineItem);
            }
            q->endInsertRows();
            break;
        }
        case 1: { // Clear
            q->beginResetModel();
            items.clear();
            q->endResetModel();
            break;
        }
        case 2: { // Push Front
            q->beginInsertRows({}, 0, 0);
            items.prepend(new RoomWrapper{diff->item()});
            q->endInsertRows();
            break;
        }
        case 3: { // Push Back
            q->beginInsertRows({}, q->rowCount({}), q->rowCount({}));
            items.prepend(new RoomWrapper{diff->item()});
            q->endInsertRows();
            break;
        }
        case 4: { // Pop Front
            q->beginRemoveRows({}, q->rowCount({}), q->rowCount({}));
            items.removeAt(0);
            q->endRemoveRows();
            break;
        }
        case 5: { // Pop Back
            q->beginRemoveRows({}, q->rowCount({}) - 1, q->rowCount({}) - 1);
            items.removeAt(q->rowCount({}) - 1);
            q->endRemoveRows();
            break;
        }
        case 6: { // Insert
            q->beginInsertRows({}, diff->index(), diff->index());
            items.insert(diff->index(), new RoomWrapper(diff->item()));
            q->endInsertRows();
            break;
        }
        case 7: { // Set
            items[diff->index()] = new RoomWrapper(diff->item());
            const auto index = q->index(diff->index(), 0);
            Q_EMIT q->dataChanged(index, index);
            break;
        }
        case 8: { // Remove
            q->beginRemoveRows({}, diff->index(), diff->index());
            items.removeAt(diff->index());
            q->endRemoveRows();
            break;
        }
        case 9: { // Truncate
            q->beginRemoveRows({}, diff->index(), q->rowCount({}) - 1);
            for (int i = diff->index(); i < q->rowCount({}); i++) {
                items.removeAt(diff->index());
            }
            q->endRemoveRows();
            break;
        }
        case 10: { // Reset
            q->beginResetModel();
            items.clear();
            auto newItems = diff->items_vec();
            for (const auto &it : newItems) {
                auto timelineItem = new RoomWrapper{it.box_me()};
                items.append(timelineItem);
            }
            q->endResetModel();
            break;
        }
    }
}

Connection *RoomsModel::connection() const
{
    return d->connection;
}

void RoomsModel::setConnection(Connection *connection)
{
    if (d->connection == connection) {
        return;
    }
    d->connection = connection;

    if (d->connection != nullptr) {
        d->roomStream = d->connection->roomStream();
        connect(d->roomStream.get(), &RoomStream::roomsUpdate, this, [this]() {
            d->roomsUpdate();
        });

        d->roomStream->startStream();
    }

    Q_EMIT connectionChanged();
}
