// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "timelinemodel.h"

#include "dispatcher.h"
#include "lib.rs.h"
#include "utils.h"

#include "connection.h"

#include <QTimer>

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
    QString m_roomId;
    std::optional<rust::Box<sdk::Timeline>> timeline;
    QList<TimelineItemWrapper *> items;
};

TimelineModel::~TimelineModel() = default;

TimelineModel::TimelineModel(QObject *parent)
    : QAbstractListModel(parent)
    , d(std::make_unique<Private>())
{
    connect(this, &TimelineModel::roomIdChanged, this, [this]() {
        if (d->connection) {
            d->timeline = d->connection->connection()->timeline(stringToRust(roomId()));
        }
    });

    connect(this, &TimelineModel::connectionChanged, this, [this]() {
        if (!d->m_roomId.isEmpty()) {
            d->connection->connection()->timeline(stringToRust(roomId()));
        }
    });
    connect(Dispatcher::instance(),
            &Dispatcher::timelineUpdate,
            this,
            [this](const auto &matrixId, const auto &roomId) {
                if (matrixId != d->connection->matrixId() || roomId != d->m_roomId) {
                    return;
                }
                timelineUpdate();
            });

    QTimer::singleShot(1000, this, [this](){
        d->connection->connection()->timeline_paginate_back(**d->timeline);
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

QString TimelineModel::roomId() const
{
    return d->m_roomId;
}

void TimelineModel::setRoomId(const QString &roomId)
{
    if (roomId == d->m_roomId) {
        return;
    }
    d->m_roomId = roomId;
    Q_EMIT roomIdChanged();
}

QHash<int, QByteArray> TimelineModel::roleNames() const
{
    return {
        {TimelineModel::IdRole, "eventId"},
        {TimelineModel::BodyRole, "body"},
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
                    case 1: { // clear
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
                        d->items.prepend(new TimelineItemWrapper{item->item()});
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
                        endInsertRows();
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
    return false; //TODO
}

void TimelineModel::fetchMore(const QModelIndex &)
{
    d->connection->connection()->timeline_paginate_back(**d->timeline);
}
