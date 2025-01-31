// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomsmodel.h"

#include "lib.rs.h"
#include "sdk/include/callbacks.h"

class RoomsModel::Private
{
public:
    App *app = nullptr;
    RoomsModel *q = nullptr;
};

RoomsModel::~RoomsModel() = default;

RoomsModel::RoomsModel()
    : QAbstractListModel(nullptr)
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
    };
}

//TODO deduplicate
static QString stringFromRust(rust::String string)
{
    return QString::fromLatin1({string.data(), (int) string.length()});
}

QVariant RoomsModel::data(const QModelIndex &index, int role) const
{
    Q_UNUSED(role);
    const auto row = index.row();

    if (role == IdRole) {
        return stringFromRust(d->app->connection()->room(row)->id()).toHtmlEscaped();
    } else if (role == DisplayNameRole) {
        return stringFromRust(d->app->connection()->room(row)->display_name()).toHtmlEscaped();
    } else if (role == AvatarUrlRole) {
        return QStringLiteral("image://roomavatar/%1").arg(stringFromRust(d->app->connection()->room(row)->id()));
    }
    return {};
}

int RoomsModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) {
        return {};
    }
    return d->app->connection()->rooms_count();
}

void shim_rooms_changed(std::uint8_t op, std::size_t from, std::size_t to)
{
    RoomsModel::instance().roomsUpdate(op, from, to);
}

void RoomsModel::roomsUpdate(std::uint8_t op, std::size_t from, std::size_t to)
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

void RoomsModel::setApp(App *app)
{
    d->app = app;
}
