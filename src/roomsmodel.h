// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QAbstractListModel>
#include <qqmlintegration.h>
#include <QQmlEngine>

#include "app.h"

class RoomsModel : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    enum RoleNames {
        IdRole = Qt::DisplayRole,
        DisplayNameRole,
    };
    Q_ENUM(RoleNames);
    static RoomsModel *create(QQmlEngine *engine, QJSEngine *)
    {
        engine->setObjectOwnership(&instance(), QQmlEngine::CppOwnership);
        return &instance();
    }
    static RoomsModel &instance() {
        static RoomsModel _instance;
        return _instance;
    };

    void setApp(App *app);

    ~RoomsModel();

    QHash<int, QByteArray> roleNames() const override;
    QVariant data(const QModelIndex &index, int role) const override;
    int rowCount(const QModelIndex &parent) const override;

    void roomsUpdate(std::uint8_t op, std::size_t from, std::size_t to);

private:
    RoomsModel();
    class Private;
    std::unique_ptr<Private> d;
};
