# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

cmake_minimum_required(VERSION 3.24)

project(QuotientDemo VERSION 0.1.0)

set(QT_MIN_VERSION "6.5")

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)

find_package(Qt6 ${QT_MIN_VERSION} REQUIRED NO_MODULE
             COMPONENTS Core Quick Gui QuickControls2 Svg Widgets)
find_package(QuotientNg REQUIRED)

if(QT_KNOWN_POLICY_QTP0001)
  qt_policy(SET QTP0001 NEW)
endif()

if(QT_KNOWN_POLICY_QTP0004)
  qt_policy(SET QTP0004 NEW)
endif()

add_subdirectory(src)
