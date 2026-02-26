// SPDX-FileCopyrightText: 2026 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "task.h"

#include "dispatcher.h"

Task::Task(const QString &token, QObject *parent)
    : QObject(parent)
    , m_token(token)
{
    //TODO connectUntil, or use Task as receiver and delete task;
    connect(Dispatcher::instance(), &Dispatcher::taskDone, this, [this, token] {
        Q_EMIT done();
        deleteLater();
    });
}