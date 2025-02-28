// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <QString>
#include <QtQmlIntegration/qqmlintegration.h>

#include "quotient_export.h"

namespace Quotient
{

class QUOTIENT_EXPORT Room : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("")

    Q_PROPERTY(QString displayName READ displayName NOTIFY displayNameChanged)
    Q_PROPERTY(QString id READ id CONSTANT)

public:
    QString displayName() const;
    QString id() const;

    ~Room();

Q_SIGNALS:
    void displayNameChanged();

private:
    friend class Connection;
    class Private;
    Room(std::unique_ptr<Private> d, QObject *parent = nullptr);
    std::unique_ptr<Private> d;
};

}
