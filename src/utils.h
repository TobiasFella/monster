// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "lib.rs.h"

#include <QString>

rust::String stringToRust(const QString &string);

QString stringFromRust(rust::String string);

rust::String stringToRust(const char *string);
