// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "roomavatarimageprovider.h"

#include "app.h"
#include "utils.h"

QQuickImageResponse *RoomAvatarImageProvider::requestImageResponse(const QString &id, const QSize &requestedSize)
{
    return new RoomAvatarImageResponse(id, requestedSize);
}

RoomAvatarImageResponse::RoomAvatarImageResponse(const QString &id, const QSize &)
{
    App::instance().connection()->room_avatar(stringToRust(id));
    connect(&App::instance(), &App::avatarLoaded, this, [id, this](const auto &roomId, const QByteArray &data) {
        if (id != roomId) {
            return;
        }
        m_image = QImage::fromData(data);
        Q_EMIT finished();
    });
}

QQuickTextureFactory *RoomAvatarImageResponse::textureFactory() const
{
    return QQuickTextureFactory::textureFactoryForImage(m_image);
}
