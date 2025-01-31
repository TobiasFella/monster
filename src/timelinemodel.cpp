// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "timelinemodel.h"

#include "lib.rs.h"
#include "sdk/include/callbacks.h"
#include "utils.h"
#include "app.h"

class TimelineModel::Private
{
public:
    TimelineModel *q = nullptr;
    QString m_roomId;
};

TimelineModel::~TimelineModel() = default;

TimelineModel::TimelineModel(QObject *parent)
: QAbstractListModel(parent)
, d(std::make_unique<Private>())
{
    d->q = this;
    connect(this, &TimelineModel::roomIdChanged, this, [this](){
        App::instance().connection()->timeline(stringToRust(roomId()));
    });
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
    };
}

QVariant TimelineModel::data(const QModelIndex &index, int role) const
{
    Q_UNUSED(role);
    const auto row = index.row();

    if (role == IdRole) {
        return stringFromRust(App::instance().connection()->timeline_item(row)->id()).toHtmlEscaped();
    }
    return {};
}

int TimelineModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) {
        return {};
    }
    return App::instance().connection()->room_event_count(stringToRust(roomId()));
}

void shim_timeline_changed(std::uint8_t op, std::size_t from, std::size_t to)
{
    TimelineModel::instance().timelineUpdate(op, from, to);
}

//TODO only react to changes to *this* room
void TimelineModel::timelineUpdate(std::uint8_t op, std::size_t from, std::size_t to)
{
    QMetaObject::invokeMethod(this, [this, op, from, to](){
        switch (op) {
            case 0: {
                beginResetModel();
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
