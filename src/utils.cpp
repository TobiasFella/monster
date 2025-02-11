// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "utils.h"

rust::String stringToRust(const QString &string)
{
    return rust::String(string.toLatin1().data(), string.length());
}

QString stringFromRust(rust::String string)
{
    return QString::fromLatin1({string.data(), (int)string.length()});
}

rust::String stringToRust(const char *string)
{
    return rust::String(string, strlen(string));
}
