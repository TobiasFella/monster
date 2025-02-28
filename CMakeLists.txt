# SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
# SPDX-License-Identifier: LGPL-2.0-or-later

cmake_minimum_required(VERSION 3.24)

project(QuotientNg VERSION 0.1.0)

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
include(ECMGenerateExportHeader)
include(ECMGenerateHeaders)

ecm_setup_version(
  PROJECT
  VARIABLE_PREFIX
  QUOTIENTNG
  VERSION_HEADER
  quotient_version.h
  PACKAGE_VERSION_FILE
  "${CMAKE_CURRENT_BINARY_DIR}/QuotientNgConfigVersion.cmake"
  SOVERSION
  0)

find_package(Qt6 ${QT_MIN_VERSION} REQUIRED NO_MODULE
             COMPONENTS Core Quick Gui QuickControls2)
find_package(Corrosion REQUIRED)
find_package(Qt6Keychain REQUIRED)

set(CMAKECONFIG_INSTALL_DIR "${KDE_INSTALL_CMAKEPACKAGEDIR}/QuotientNg")

configure_package_config_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/QuotientNgConfig.cmake.in"
  "${CMAKE_CURRENT_BINARY_DIR}/QuotientNgConfig.cmake"
  INSTALL_DESTINATION "${CMAKECONFIG_INSTALL_DIR}")

install(
  FILES "${CMAKE_CURRENT_BINARY_DIR}/QuotientNgConfig.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/QuotientNgConfigVersion.cmake"
  DESTINATION "${CMAKECONFIG_INSTALL_DIR}"
  COMPONENT Devel)

install(
  EXPORT QuotientNgTargets
  DESTINATION "${CMAKECONFIG_INSTALL_DIR}"
  FILE QuotientNgTargets.cmake)

install(
  FILES ${CMAKE_CURRENT_BINARY_DIR}/quotient_version.h
  DESTINATION ${KDE_INSTALL_INCLUDEDIR}
  COMPONENT Devel)

if(QT_KNOWN_POLICY_QTP0004)
  qt_policy(SET QTP0004 NEW)
endif()

corrosion_import_crate(MANIFEST_PATH src/sdk/Cargo.toml)

install(TARGETS sdk EXPORT QuotientNgTargets)

add_subdirectory(src)

file(GLOB_RECURSE ALL_CLANG_FORMAT_SOURCE_FILES src/*.cpp src/*.h)
kde_clang_format(${ALL_CLANG_FORMAT_SOURCE_FILES})
kde_configure_git_pre_commit_hook(CHECKS CLANG_FORMAT)
