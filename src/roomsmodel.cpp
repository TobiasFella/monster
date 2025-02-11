// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomsmodel.h"

#include "connection.h"
#include "dispatcher.h"
#include "lib.rs.h"
#include "utils.h"

#include <QPointer>

class RoomsModel::Private
{
public:
    QPointer<Connection> connection;
    std::optional<rust::Box<sdk::Rooms>> rooms;
};

RoomsModel::~RoomsModel() = default;

RoomsModel::RoomsModel(QObject *parent)
    : QAbstractListModel(parent)
    , d(std::make_unique<Private>())
{
    connect(this, &RoomsModel::connectionChanged, this, [this]() {
        d->rooms = d->connection->connection()->slide();
    });
    connect(Dispatcher::instance(), &Dispatcher::roomsUpdate, this, [this](const auto &matrixId, const auto op, const auto from, const auto to) {
        if (matrixId != d->connection->matrixId()) {
            return;
        }
        roomsUpdate(op, from, to);
    });
}

QHash<int, QByteArray> RoomsModel::roleNames() const
{
    return {
        {RoomsModel::IdRole, "roomId"},
        {RoomsModel::DisplayNameRole, "displayName"},
        {RoomsModel::AvatarUrlRole, "avatarUrl"},
    };
}

QVariant RoomsModel::data(const QModelIndex &index, int role) const
{
    Q_UNUSED(role);
    const auto row = index.row();
    if (row >= (int)(*d->rooms)->count()) {
        // TODO why
        return {};
    }

    if (role == IdRole) {
        return stringFromRust((*d->rooms)->room(row)->id()).toHtmlEscaped();
    } else if (role == DisplayNameRole) {
        return stringFromRust((*d->rooms)->room(row)->display_name()).toHtmlEscaped();
    } else if (role == AvatarUrlRole) {
        return QStringLiteral("image://roomavatar/%1").arg(stringFromRust((*d->rooms)->room(row)->id()));
    }
    return {};
}

int RoomsModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) {
        return {};
    }
    return (*d->rooms)->count();
}

void RoomsModel::roomsUpdate(std::uint8_t op, std::size_t from, std::size_t to)
{
    QMetaObject::invokeMethod(
        this,
        [this, op, from, to]() {
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
