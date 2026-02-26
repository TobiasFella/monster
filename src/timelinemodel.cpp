// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "timelinemodel.h"

#include "dispatcher.h"
#include "ffi.rs.h"
#include "utils.h"

#include "connection.h"

using namespace Quotient;

struct TimelineItemWrapper
{
    std::optional<rust::Box<sdk::TimelineItem>> item;
};

class TimelineModel::Private
{
public:
    ~Private() {
        for (const auto &item : items) {
            delete item;
        }
    }

public:
    QPointer<Connection> connection;
    QPointer<Room> room;
    std::optional<rust::Box<sdk::Timeline>> timeline;
    QList<TimelineItemWrapper *> items;
};

TimelineModel::~TimelineModel() = default;

TimelineModel::TimelineModel(QObject *parent)
    : QAbstractListModel(parent)
    , d(std::make_unique<Private>())
{
    connect(this, &TimelineModel::roomChanged, this, [this]() {
        if (d->connection) {
            d->timeline = d->connection->connection()->timeline(stringToRust(room()->id()));
        }
    });

    connect(this, &TimelineModel::connectionChanged, this, [this]() {
        if (d->room) {
            d->timeline = d->connection->connection()->timeline(stringToRust(room()->id()));
        }
    });
    connect(Dispatcher::instance(),
            &Dispatcher::timelineUpdate,
            this,
            [this](const auto &matrixId, const auto &roomId) {
                if (matrixId != d->connection->matrixId() || roomId != room()->id()) {
                    return;
                }
                timelineUpdate();
            });
}

Connection *TimelineModel::connection() const
{
    return d->connection;
}

void TimelineModel::setConnection(Connection *connection)
{
    if (connection == d->connection) {
        return;
    }
    d->connection = connection;
    Q_EMIT connectionChanged();
}

Room *TimelineModel::room() const
{
    return d->room;
}

void TimelineModel::setRoom(Room *room)
{
    if (room == d->room) {
        return;
    }
    d->room = room;
    Q_EMIT roomChanged();
}

QHash<int, QByteArray> TimelineModel::roleNames() const
{
    return {
        {TimelineModel::IdRole, "eventId"},
        {TimelineModel::BodyRole, "body"},
        {TimelineModel::TimestampRole, "timestamp"},
    };
}

QVariant TimelineModel::data(const QModelIndex &index, int role) const
{
    Q_UNUSED(role);
    const auto row = index.row();

    if (role == IdRole) {
        return stringFromRust((*d->items[row]->item)->id());
    }
    if (role == BodyRole) {
        return stringFromRust((*d->items[row]->item)->body());
    }
    if (role == TimestampRole) {
        return stringFromRust((*d->items[row]->item)->timestamp());
    }
    return {};
}

int TimelineModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid() || !d->timeline) {
        return {};
    }
    return d->items.size();
}

void TimelineModel::timelineUpdate()
{
    QMetaObject::invokeMethod(
        this,
        [this]() {
            while ((*d->timeline)->has_queued_item()) {
                auto item = (*d->timeline)->queue_next();
                switch (item->op()) {
                    case 0: { // Append
                        auto items = item->items_vec();
                        beginInsertRows({}, rowCount({}), rowCount({}) + items.size());
                        for (const auto &it : items) {
                            auto timelineItem = new TimelineItemWrapper{it.box_me()};
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
                        d->items.prepend(new TimelineItemWrapper{item->item()});
                        endInsertRows();
                        break;
                    }
                    case 3: { // Push Back
                        beginInsertRows({}, rowCount({}), rowCount({}));
                        d->items.append(new TimelineItemWrapper{item->item()});
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
                        d->items.insert(item->index(), new TimelineItemWrapper(item->item()));
                        endInsertRows();
                        break;
                    }
                    case 7: { // Set
                        d->items[item->index()] = new TimelineItemWrapper(item->item());
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
                            auto timelineItem = new TimelineItemWrapper{it.box_me()};
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

bool TimelineModel::canFetchMore(const QModelIndex &) const
{
    return room();
}

void TimelineModel::fetchMore(const QModelIndex &)
{
    if (room()) {
        d->connection->connection()->timeline_paginate_back(**d->timeline);
    }
}

ReversedTimelineModel::ReversedTimelineModel(QObject *parent)
    : QSortFilterProxyModel(parent)
{
    sort(0);
}

bool ReversedTimelineModel::lessThan(const QModelIndex &sourceLeft, const QModelIndex &sourceRight) const
{
    return sourceLeft.row() > sourceRight.row();
}

void TimelineModel::sendMessage(const QString &message)
{
    (*d->timeline)->send_message(*d->connection->connection(), stringToRust(message));
}
