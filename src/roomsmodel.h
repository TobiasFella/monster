// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <qqmlintegration.h>
#include <QQmlEngine>

class Connection;

class RoomsModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(Connection *connection READ connection WRITE setConnection NOTIFY connectionChanged)

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
        DisplayNameRole,
        AvatarUrlRole,
    };
    Q_ENUM(RoleNames);

    RoomsModel(QObject *parent = nullptr);
    ~RoomsModel();

    void setConnection(Connection *connection);
    Connection *connection() const;

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    void roomsUpdate(std::uint8_t op, std::size_t from, std::size_t to);

Q_SIGNALS:
    void connectionChanged();

private:
    class Private;
    std::unique_ptr<Private> d;
};
