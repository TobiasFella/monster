// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "timelinemodel.h"

#include "dispatcher.h"
#include "lib.rs.h"
#include "utils.h"

#include "connection.h"

class TimelineModel::Private
{
public:
    QPointer<Connection> connection;
    QString m_roomId;
    std::optional<rust::Box<sdk::Timeline>> timeline;
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
    connect(Dispatcher::instance(), &Dispatcher::timelineUpdate, this, [this](const auto &matrixId, const auto &roomId, const auto op, const auto from, const auto to) {
        if (matrixId != d->connection->matrixId() || roomId != d->m_roomId) {
            return;
        }
        timelineUpdate(op, from, to);
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
        return stringFromRust((*d->timeline)->timeline_item(row)->id()).toHtmlEscaped();
    }
    if (role == BodyRole) {
        return stringFromRust((*d->timeline)->timeline_item(row)->body());
    }
    return {};
}

int TimelineModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid() || !d->timeline) {
        return {};
    }
    return (*d->timeline)->count();
}

void TimelineModel::timelineUpdate(std::uint8_t op, std::size_t from, std::size_t to)
{
    QMetaObject::invokeMethod(this, [this, op, from, to](){
        switch (op) {
            case 0: {
                beginInsertRows({}, from, to);
                endInsertRows();
                break;
            }
            case 1: {
                beginResetModel();
                endResetModel();
                break;
            }
            case 2: {
                beginInsertRows({}, from, to);
                endInsertRows();
                break;
            }
            case 3: {
                beginInsertRows({}, from, to);
                endInsertRows();
                break;
            }
            case 4: {
                beginRemoveRows({}, from, to);
                endRemoveRows();
                break;
            }
            case 5: {
                beginRemoveRows({}, from, to);
                endRemoveRows();
                break;
            }
            case 6: {
                beginInsertRows({}, from, to);
                endInsertRows();
                break;
            }
            case 7: {
                Q_EMIT dataChanged(index(from, 0), index(to, 0));
                break;
            }
            case 8: {
                beginRemoveRows({}, from, to);
                endRemoveRows();
                break;
            }
            case 9: {
                beginRemoveRows({}, from, to);
                endRemoveRows();
                break;
            }
            case 10: {
                beginResetModel();
                endResetModel();
                break;
            }
        }
    }, Qt::QueuedConnection);
}


bool TimelineModel::canFetchMore(const QModelIndex &) const
{
    return true;
}

void TimelineModel::fetchMore(const QModelIndex &)
{
    d->connection->connection()->timeline_paginate_back(**d->timeline);
}

ReversedTimelineModel::ReversedTimelineModel(QObject *parent)
    :QSortFilterProxyModel(parent)
{
    sort(0);
}

bool ReversedTimelineModel::lessThan(const QModelIndex &sourceLeft, const QModelIndex &sourceRight) const
{
    return !QSortFilterProxyModel::lessThan(sourceLeft, sourceRight);
}
