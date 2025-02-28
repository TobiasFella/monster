// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#if __has_include("lib.rs.h")
#include "lib.rs.h"
#else
#include <QuotientNg/lib.rs.h>
#endif

#include <QString>

#include "quotient_export.h"

QUOTIENT_EXPORT rust::String stringToRust(const QString &string);

QUOTIENT_EXPORT QString stringFromRust(rust::String string);

QUOTIENT_EXPORT rust::String stringToRust(const char *string);
