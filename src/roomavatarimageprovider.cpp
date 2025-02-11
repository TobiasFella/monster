// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomavatarimageprovider.h"

#include "connection.h"
#include "dispatcher.h"
#include "utils.h"

RoomAvatarImageProvider::RoomAvatarImageProvider() = default;

QQuickImageResponse *RoomAvatarImageProvider::requestImageResponse(const QString &id, const QSize &requestedSize)
{
    return new RoomAvatarImageResponse(id, requestedSize, m_connection.get());
}

RoomAvatarImageResponse::RoomAvatarImageResponse(const QString &id, const QSize &, Connection *connection)
{
    connection->connection()->room_avatar(stringToRust(id));
    connect(Dispatcher::instance(), &Dispatcher::avatarLoaded, this, [id, this](const auto &roomId, const QByteArray &data) {
        if (id != roomId) {
            return;
        }
        m_image = QImage::fromData(data);
        Q_EMIT finished();
    });
}

Connection *RoomAvatarImageProvider::connection() const
{
    return m_connection;
}

void RoomAvatarImageProvider::setConnection(Connection *connection)
{
    if (m_connection == connection) {
        return;
    }
    m_connection = connection;
    Q_EMIT connectionChanged();
}

QQuickTextureFactory *RoomAvatarImageResponse::textureFactory() const
{
    return QQuickTextureFactory::textureFactoryForImage(m_image);
}
