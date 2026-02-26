// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "utils.h"

rust::String stringToRust(const QString &string)
{
    return {string.toLatin1().data(), string.length()};
}

QString stringFromRust(const rust::String &string)
{
    return QString::fromLatin1({string.data(), static_cast<int>(string.length())});
}

rust::String stringToRust(const char *string)
{
    return {string, strlen(string)};
}
QByteArray bytesFromRust(const rust::String &string)
{
    return {string.data(), static_cast<qsizetype>(string.length())};
}
