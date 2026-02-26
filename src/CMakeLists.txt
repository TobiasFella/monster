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
    accounts.cpp
    pendingconnection.cpp
    room.cpp
    roomstream.cpp
    roomtreeitem.cpp
    roomtreemodel.cpp
    sortfilterroomtreemodel.cpp
    roomsortparameter.cpp
    sdk/src/task.cpp
    sdk/src/task.h
)

ecm_add_qml_module(monster URI im.arctic.monster GENERATE_PLUGIN_SOURCE
    QML_FILES
        qml/Main.qml
        qml/RoomListPage.qml
        qml/RoomPage.qml
        qml/RoomDelegate.qml
        qml/RoomTreeSection.qml
        qml/AvatarNotification.qml
    DEPENDENCIES
        QtCore
)

target_link_libraries(monster PRIVATE sdk Qt::Core Qt::Gui Qt::Qml Qt::Quick Qt::Widgets KF6::I18n qt6keychain)

string(SHA1 cargo_path_hash ${CMAKE_CURRENT_SOURCE_DIR}/sdk/Cargo.toml)
string(SUBSTRING "${cargo_path_hash}" 0 5 cargo_path_hash)
set(corrosion_build_dir "sdk_${cargo_path_hash}")
target_include_directories(monster PUBLIC
        ${CMAKE_BINARY_DIR}/cargo/${corrosion_build_dir}/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/
        ${CMAKE_BINARY_DIR}/cargo/${corrosion_build_dir}/${Rust_CARGO_TARGET}/cxxbridge/
)
install(TARGETS monster ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})
