// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>
#include <QSortFilterProxyModel>
#include <qqmlintegration.h>

class Connection;

class TimelineModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString roomId READ roomId WRITE setRoomId NOTIFY roomIdChanged)
    Q_PROPERTY(Connection *connection READ connection WRITE setConnection NOTIFY connectionChanged)

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
        BodyRole,
    };
    Q_ENUM(RoleNames);

    TimelineModel(QObject *parent = nullptr);
    ~TimelineModel();

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    bool canFetchMore(const QModelIndex &parent = {}) const override;
    void fetchMore(const QModelIndex &parent = {}) override;

    QString roomId() const;
    void setRoomId(const QString &roomId);

    Connection *connection() const;
    void setConnection(Connection *connection);

    Q_INVOKABLE void fetch();

Q_SIGNALS:
    void roomIdChanged();
    void connectionChanged();

private:
    class Private;
    std::unique_ptr<Private> d;
    void timelineUpdate();
};
