// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#pragma once

#if __has_include("lib.rs.h")
#include "lib.rs.h"
#else
#include <QuotientNg/lib.rs.h>
#endif

#include "connection.h"

using namespace Quotient;

struct RustConnectionWrapper {
    std::optional<rust::Box<sdk::Connection>> m_connection;
};

class Connection::Private
{
public:
    RustConnectionWrapper *wrapper = nullptr;
    rust::Box<sdk::Connection> &connection() const;

    Private(RustConnectionWrapper *wrapper)
        : wrapper(wrapper)
    {
    }

    ~Private()
    {
        delete wrapper;
    }
};
