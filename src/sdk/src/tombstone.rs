// SPDX-FileCopyrightText: 2025 James Graham <james.h.graham@protonmail.com>
// SPDX-License-Identifier: LGPL-2.0-or-later

pub struct RoomTombstoneEventContent(pub Option<matrix_sdk::ruma::events::room::tombstone::RoomTombstoneEventContent>);

impl RoomTombstoneEventContent {
    pub fn body(&self) -> String {
        match self.0.as_ref() {
            None => Default::default(),
            Some(inner) => inner.body.to_string(),
        }
    }

    pub fn replacement_room(&self) -> String {
        match self.0.as_ref() {
            None => Default::default(),
            Some(inner) => inner.replacement_room.to_string(),
        }
    }

    pub fn empty() -> RoomTombstoneEventContent {
        RoomTombstoneEventContent(None)
    }
}
