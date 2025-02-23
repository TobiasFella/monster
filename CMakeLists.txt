# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

cmake_minimum_required(VERSION 3.24)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(Monster VERSION 0.1.0)

set(KF_MIN_VERSION "6.6")
set(QT_MIN_VERSION "6.5")

find_package(ECM ${KF_MIN_VERSION} REQUIRED NO_MODULE)

set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH})

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(KDE_COMPILERSETTINGS_LEVEL 6.0)

include(FeatureSummary)
include(ECMSetupVersion)
include(KDEInstallDirs)
include(KDECMakeSettings)
include(KDECompilerSettings NO_POLICY_SCOPE)
include(KDEGitCommitHooks)
include(KDEClangFormat)
include(ECMQmlModule)

find_package(Qt6 ${QT_MIN_VERSION} REQUIRED NO_MODULE COMPONENTS Core Quick Gui QuickControls2 Svg Widgets)
find_package(KF6 ${KF_MIN_VERSION} REQUIRED COMPONENTS I18n)
find_package(Corrosion REQUIRED)
find_package(Qt6Keychain REQUIRED)

# if (QT_KNOWN_POLICY_QTP0004)
#     qt_policy(SET QTP0004 NEW)
# endif ()

corrosion_import_crate(MANIFEST_PATH src/sdk/Cargo.toml)

add_subdirectory(src)
