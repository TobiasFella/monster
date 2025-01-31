// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QQuickAsyncImageProvider>

#include "app.h"

class RoomAvatarImageProvider : public QQuickAsyncImageProvider
{
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override;
};

class RoomAvatarImageResponse : public QQuickImageResponse
{
    Q_OBJECT
public:
    RoomAvatarImageResponse(const QString &id, const QSize &requestedSize);
    QQuickTextureFactory *textureFactory() const override;
private:
    QImage m_image;
};
