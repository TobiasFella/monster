// SPDX-FileCopyrightText: Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: BSD-2-Clause

use cxx_build::CFG;

fn main() {
    CFG.include_prefix = "sdk";
    cxx_build::bridge("src/lib.rs").std("c++20").compile("sdk");
}
