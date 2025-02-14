// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QQuickAsyncImageProvider>

#include "connection.h"

class RoomAvatarImageProvider : public QQuickAsyncImageProvider
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(Quotient::Connection *connection READ connection WRITE setConnection NOTIFY connectionChanged)
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override;

    static RoomAvatarImageProvider *instance()
    {
        static RoomAvatarImageProvider *_instance = new RoomAvatarImageProvider;
        return _instance;
    }

    static RoomAvatarImageProvider *create(QQmlEngine *, QJSEngine *)
    {
        QQmlEngine::setObjectOwnership(instance(), QQmlEngine::CppOwnership);
        return instance();
    }

    Quotient::Connection *connection() const;
    void setConnection(Quotient::Connection *connection);

Q_SIGNALS:
    void connectionChanged();

private:
    RoomAvatarImageProvider();
    QPointer<Quotient::Connection> m_connection;
};

class RoomAvatarImageResponse : public QQuickImageResponse
{
    Q_OBJECT
public:
    RoomAvatarImageResponse(const QString &id, const QSize &requestedSize, Quotient::Connection *connection);
    QQuickTextureFactory *textureFactory() const override;

private:
    QImage m_image;
};
