# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

#add_executable(monster
#    main.cpp)

add_library(QuotientNg
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
)

# ecm_add_qml_module(monster URI im.arctic.monster GENERATE_PLUGIN_SOURCE
#     QML_FILES
#         qml/Main.qml
#         qml/RoomListPage.qml
#         qml/RoomPage.qml
#     DEPENDENCIES
#         QtCore
# )

target_sources(QuotientNg PUBLIC FILE_SET HEADERS TYPE HEADERS BASE_DIRS ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/
    FILES
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/lib.rs.h
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/rust/cxx.h
)

target_link_libraries(QuotientNg PUBLIC sdk Qt::Core Qt::Gui Qt::Qml Qt::Quick Qt::Widgets KF6::I18n qt6keychain)
target_include_directories(QuotientNg PRIVATE
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/
)

install(TARGETS QuotientNg EXPORT QuotientNgTargets
    RUNTIME LIBRARY ARCHIVE FILE_SET HEADERS)

write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/QuotientNg/QuotientNgConfigVersion.cmake"
    COMPATIBILITY SameMajorVersion
)

export(PACKAGE QuotientNg)
export(EXPORT QuotientNgTargets
       FILE "${CMAKE_CURRENT_BINARY_DIR}/QuotientNg/QuotientNgTargets.cmake")

if (NOT DEFINED BUILD_SHARED_LIBS)
    set(BUILD_SHARED_LIBS OFF)
endif()
configure_file(../cmake/QuotientNgConfig.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/QuotientNg/QuotientNgConfig.cmake"
    @ONLY
)

install(EXPORT QuotientNgTargets
        FILE QuotientNgTargets.cmake DESTINATION ${KDE_INSTALL_CMAKEPACKAGEDIR}/QuotientNg)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/QuotientNg/QuotientNgConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/QuotientNg/QuotientNgConfigVersion.cmake"
    DESTINATION ${KDE_INSTALL_CMAKEPACKAGEDIR}/QuotientNg
)

# target_link_libraries(monster PRIVATE QuotientNg)

# install(TARGETS monster ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})
