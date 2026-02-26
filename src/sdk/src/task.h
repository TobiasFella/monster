// SPDX-FileCopyrightText: 2026 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#include <QObject>
#include <qqmlintegration.h>

class Task : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("Returned when calling functions running tasks")
public:
    explicit Task(const QString &token, QObject *parent = nullptr);

Q_SIGNALS:
    void done();
private:
    QString m_token;
};
