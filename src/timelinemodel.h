// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <qqmlintegration.h>
#include <QQmlEngine>

class TimelineModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(QString roomId READ roomId WRITE setRoomId NOTIFY roomIdChanged)

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
    };
    Q_ENUM(RoleNames);
    static TimelineModel *create(QQmlEngine *engine, QJSEngine *)
    {
        engine->setObjectOwnership(&instance(), QQmlEngine::CppOwnership);
        return &instance();
    }
    static TimelineModel &instance() {
        static TimelineModel _instance;
        return _instance;
    };

    ~TimelineModel();

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    QString roomId() const;
    void setRoomId(const QString &roomId);

    void timelineUpdate(std::uint8_t op, std::size_t from, std::size_t to);

Q_SIGNALS:
    void roomIdChanged();

private:
    TimelineModel(QObject *parent = nullptr);
    class Private;
    std::unique_ptr<Private> d;
};
