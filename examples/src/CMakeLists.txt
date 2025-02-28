# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: BSD-2-Clause

add_executable(QuotientDemo main.cpp foreigntypes.h timelinemodel.cpp
                            roomsmodel.cpp roomavatarimageprovider.cpp)

qt_add_qml_module(
  QuotientDemo
  URI
  org.kde.quotient.demo
  QML_FILES
  qml/Main.qml
  qml/RoomListPage.qml
  qml/RoomPage.qml
  DEPENDENCIES
  QtCore)

target_link_libraries(QuotientDemo PRIVATE QuotientNg Qt::Gui Qt::Quick)

install(TARGETS QuotientDemo ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})
