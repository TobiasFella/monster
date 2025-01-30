<!-- SPDX-FileCopyrightText: Tobias Fella <tobias.fella@kde.org>
     SPDX-License-Identifier: CC0-1.0 -->

# Monster

An experimental Matrix client written in Qt using matrix-rust-sdk. The main focus is on exploring how to use asynchronous rust SDKs from a C++/Qt application.

## Building
```bash
cmake -B build
cmake --build build
cmake --install build
```

## Dependencies

- Qt6 Base, Declarative
- KF6 I18n
- Corrosion
- Reasonably new rustc, cargo

## License

All code is licensed as stated in the SPDX headers.
