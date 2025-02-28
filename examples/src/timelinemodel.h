// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <QSortFilterProxyModel>
#include <QtQmlIntegration/qqmlintegration.h>

#include <QuotientNg/Room>

#include "quotient_export.h"

namespace Quotient
{
class Connection;
};

class QUOTIENT_EXPORT TimelineModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(Quotient::Room *room READ room WRITE setRoom NOTIFY roomChanged)
    Q_PROPERTY(Quotient::Connection *connection READ connection WRITE setConnection NOTIFY connectionChanged)

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
        BodyRole,
        TimestampRole,
    };
    Q_ENUM(RoleNames);

    TimelineModel(QObject *parent = nullptr);
    ~TimelineModel();

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    bool canFetchMore(const QModelIndex &parent = {}) const override;
    void fetchMore(const QModelIndex &parent = {}) override;

    Quotient::Room *room() const;
    void setRoom(Quotient::Room *room);

    Quotient::Connection *connection() const;
    void setConnection(Quotient::Connection *connection);

    Q_INVOKABLE void sendMessage(const QString &message);

Q_SIGNALS:
    void roomChanged();
    void connectionChanged();

private:
    class Private;
    std::unique_ptr<Private> d;
    void timelineUpdate();
};

class ReversedTimelineModel : public QSortFilterProxyModel
{
    Q_OBJECT
    QML_ELEMENT
public:
    explicit ReversedTimelineModel(QObject *parent = nullptr);

    bool lessThan(const QModelIndex &sourceLeft, const QModelIndex &sourceRight) const override;
};
