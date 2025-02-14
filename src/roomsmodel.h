// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>
#include <qqmlintegration.h>

namespace Quotient
{
class Connection;
}

class RoomsModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(Quotient::Connection *connection READ connection WRITE setConnection NOTIFY connectionChanged)

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
        DisplayNameRole,
        AvatarUrlRole,
    };
    Q_ENUM(RoleNames);

    RoomsModel(QObject *parent = nullptr);
    ~RoomsModel();

    void setConnection(Quotient::Connection *connection);
    Quotient::Connection *connection() const;

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    void roomsUpdate();

Q_SIGNALS:
    void connectionChanged();

private:
    class Private;
    std::unique_ptr<Private> d;
};
