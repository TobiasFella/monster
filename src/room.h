// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QString>
#include <qqmlintegration.h>

#include "lib.rs.h"
#include "rust/cxx.h"

namespace Quotient
{

class Room : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

    Q_PROPERTY(QString displayName READ displayName NOTIFY displayNameChanged)
    Q_PROPERTY(QString id READ id CONSTANT)

public:
    QString id() const;
    rust::u8 state() const;
    bool isSpace() const;
    QString roomType() const;
    QString displayName() const;
    bool isTombstoned() const;
    rust::Box<sdk::RoomTombstoneEventContent> tombstone() const;
    QString topic() const;
    int numUnreadMessages() const;
    int numUnreadMentions() const;
    bool isFavourite() const;
    bool isLowPriority() const;

    ~Room();

Q_SIGNALS:
    void displayNameChanged();

private:
    friend class Connection;
    Room(rust::Box<sdk::Room> room, QObject *parent = nullptr);
    class Private;
    std::unique_ptr<Private> d;
};

}
