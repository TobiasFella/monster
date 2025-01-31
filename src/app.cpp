// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

#include "app.h"

#include <QDebug>

#include "roomsmodel.h"
#include "utils.h"

class App::Private
{
public:
    std::optional<rust::Box<sdk::Connection>> m_connection;
};

App::~App() = default;

App::App()
    : QObject(nullptr)
    , d(std::make_unique<Private>())
{
    RoomsModel::instance().setApp(this);
}

//TODO: Simplify once we can better use callbacks
// "Passing a function pointer from C++ to Rust is not implemented yet, only from Rust to an extern "C++" function is implemented."
//TODO: Figure out a way of mapping requests to callback invocations. uuids/tokens/ids? user_data pointer w/ qobject?
void shim_connected() {
    Q_EMIT App::instance().connected();
}

void App::login(const QString &matrixId, const QString &password)
{
    d->m_connection = sdk::init(stringToRust(matrixId), stringToRust(password));
    connect(this, &App::connected, this, [this](){
        m_loggedIn = true;
        Q_EMIT loggedInChanged();
        (*d->m_connection)->slide();
    }, Qt::SingleShotConnection);
}

rust::Box<sdk::Connection> &App::connection() const
{
    return *d->m_connection;
}

void shim_avatar_loaded(rust::String roomId, rust::Vec<std::uint8_t> _data)
{
    QByteArray data((const char *) _data.data(), _data.size());
    Q_EMIT App::instance().avatarLoaded(stringFromRust(roomId), data);
}
