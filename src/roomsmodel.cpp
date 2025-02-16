// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomsmodel.h"

#include "connection.h"
#include "dispatcher.h"
#include "lib.rs.h"
#include "utils.h"

#include <QPointer>

using namespace Quotient;

struct RoomWrapper
{
    std::optional<rust::Box<sdk::RoomListRoom>> item;
};

class RoomsModel::Private
{
public:
    QPointer<Quotient::Connection> connection;
    std::optional<rust::Box<sdk::Rooms>> rooms;
    QList<RoomWrapper *> items;

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
    connect(this, &RoomsModel::connectionChanged, this, [this]() {
        d->rooms = d->connection->connection()->slide();
    });
    connect(Dispatcher::instance(), &Dispatcher::roomsUpdate, this, [this](const auto &matrixId) {
        if (matrixId != d->connection->matrixId()) {
            return;
        }
        roomsUpdate();
    });
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

void RoomsModel::roomsUpdate()
{
    QMetaObject::invokeMethod(
        this,
        [this]() {
            while ((*d->rooms)->has_queued_item()) {
                auto item = (*d->rooms)->queue_next();
                switch (item->op()) {
                    case 0: { // Append
                        auto items = item->items_vec();
                        beginInsertRows({}, rowCount({}), rowCount({}) + items.size());
                        for (const auto &it : items) {
                            auto timelineItem = new RoomWrapper{it.box_me()};
                            d->items.append(timelineItem);
                        }
                        endInsertRows();
                        break;
                    }
                    case 1: { // Clear
                        beginResetModel();
                        d->items.clear();
                        endResetModel();
                        break;
                    }
                    case 2: { // Push Front
                        beginInsertRows({}, 0, 0);
                        d->items.prepend(new RoomWrapper{item->item()});
                        endInsertRows();
                        break;
                    }
                    case 3: { // Push Back
                        beginInsertRows({}, rowCount({}), rowCount({}));
                        d->items.prepend(new RoomWrapper{item->item()});
                        endInsertRows();
                        break;
                    }
                    case 4: { // Pop Front
                        beginRemoveRows({}, rowCount({}), rowCount({}));
                        d->items.removeAt(0);
                        endRemoveRows();
                        break;
                    }
                    case 5: { // Pop Back
                        beginRemoveRows({}, rowCount({}) - 1, rowCount({}) - 1);
                        d->items.removeAt(rowCount({}) - 1);
                        endRemoveRows();
                        break;
                    }
                    case 6: { // Insert
                        beginInsertRows({}, item->index(), item->index());
                        d->items.insert(item->index(), new RoomWrapper(item->item()));
                        endInsertRows();
                        break;
                    }
                    case 7: { // Set
                        d->items[item->index()] = new RoomWrapper(item->item());
                        Q_EMIT dataChanged(index(item->index(), 0), index(item->index(), 0));
                        break;
                    }
                    case 8: { // Remove
                        beginRemoveRows({}, item->index(), item->index());
                        d->items.removeAt(item->index());
                        endRemoveRows();
                        break;
                    }
                    case 9: { // Truncate
                        beginRemoveRows({}, item->index(), rowCount({}) - 1);
                        for (int i = item->index(); i < rowCount({}); i++) {
                            d->items.removeAt(item->index());
                        }
                        endRemoveRows();
                        break;
                    }
                    case 10: { // Reset
                        beginResetModel();
                        d->items.clear();
                        auto items = item->items_vec();
                        for (const auto &it : items) {
                            auto timelineItem = new RoomWrapper{it.box_me()};
                            d->items.append(timelineItem);
                        }
                        endResetModel();
                        break;
                    }
                }
            }
        },
        Qt::QueuedConnection);
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
    Q_EMIT connectionChanged();
}
