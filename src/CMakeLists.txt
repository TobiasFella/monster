# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

add_executable(monster
    main.cpp
    connection.cpp
    roomsmodel.cpp
    roomavatarimageprovider.cpp
    utils.cpp
    timelinemodel.cpp
    dispatcher.cpp
)

ecm_add_qml_module(monster URI im.arctic.monster GENERATE_PLUGIN_SOURCE
    QML_FILES
        qml/Main.qml
        qml/RoomListPage.qml
        qml/RoomPage.qml
    DEPENDENCIES
        QtCore
)

target_link_libraries(monster PRIVATE sdk Qt::Core Qt::Gui Qt::Qml Qt::Quick Qt::Widgets KF6::I18n qt6keychain)

target_include_directories(monster PUBLIC
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/
)
install(TARGETS monster ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})
