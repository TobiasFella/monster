# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

add_library(QuotientNg connection.cpp utils.cpp dispatcher.cpp accounts.cpp
                       pendingconnection.cpp room.cpp roomstream.cpp
)

ecm_generate_export_header(QuotientNg BASE_NAME quotient VERSION
                           ${CMAKE_PROJECT_VERSION})
qt_extract_metatypes(QuotientNg)

configure_file(copy_header.sh.in ${CMAKE_BINARY_DIR}/copy_header.sh)
add_custom_target(copy_header ALL COMMAND sh -c ${CMAKE_BINARY_DIR}/copy_header.sh)
add_dependencies(copy_header QuotientNg)

target_link_libraries(
  QuotientNg
  PUBLIC Qt::Core qt6keychain
  PRIVATE sdk)

set_target_properties(
  QuotientNg
  PROPERTIES VERSION ${CMAKE_PROJECT_VERSION}
             SOVERSION 0
             EXPORT_NAME QuotientNg)

target_include_directories(
  QuotientNg
  PRIVATE
    "$<BUILD_INTERFACE:${QuotientNg_SOURCE_DIR}/src;${QuotientNg_BINARY_DIR}/src>"
    ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/
  INTERFACE "$<INSTALL_INTERFACE:${KDE_INSTALL_INCLUDEDIR}/QuotientNg>")

ecm_generate_headers(
  QuotientNg_CamelCase_HEADERS
  HEADER_NAMES
  PendingConnection
  Connection
  Room
  Accounts
  Dispatcher
  Utils
  Connection_p
  PREFIX
  QuotientNg
  REQUIRED_HEADERS
  QuotientNg_HEADERS)

install(
  TARGETS QuotientNg
  EXPORT QuotientNgTargets
  ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})

install(
  FILES ${QuotientNg_HEADERS}
  DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg/quotientng
  COMPONENT Devel)

install(
  FILES ${CMAKE_CURRENT_BINARY_DIR}/quotient_export.h
  DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg
  COMPONENT Devel)

install(
  FILES ${QuotientNg_CamelCase_HEADERS}
  DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg/QuotientNg
  COMPONENT Devel)

install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/sdk/include/callbacks.h"
        DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg/sdk/include)

get_filename_component(
  RUST_HEADER_DIR
  ${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}/cxxbridge/sdk/src/lib.rs.h
  REALPATH)

message(${RUST_HEADER_DIR})
install(FILES ${RUST_HEADER_DIR}
        DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg/QuotientNg)

install(FILES ${CMAKE_BINARY_DIR}/lib.rs.h DESTINATION ${KDE_INSTALL_INCLUDEDIR}/QuotientNg/QuotientNg)
