// SPDX-FileCopyrightText: 2025 James Graham <james.h.graham@protonmail.com>
// SPDX-License-Identifier: LGPL-2.0-or-later

use matrix_sdk::{
    ruma::room::RoomType,
    RoomState,
};

use crate::tombstone::RoomTombstoneEventContent;

pub struct Room {
    pub room: matrix_sdk::room::Room,
}

impl Room {
    pub fn id(&self) -> String {
        self.room.room_id().to_string()
    }

    /// Get the state of the room.
    pub fn state(&self) -> u8 {
        match self.room.state() {
            RoomState::Joined { .. } => 0,
            RoomState::Left { .. } => 1,
            RoomState::Invited { .. } => 2,
            RoomState::Knocked { .. } => 3,
            RoomState::Banned { .. } => 4,
        }
    }

    /// Whether this room's [`RoomType`] is `m.space`.
    pub fn is_space(&self) -> bool {
        self.room.is_space()
    }

    /// Returns the room's type as defined in its creation event
    /// (`m.room.create`).
    pub fn room_type(&self) -> String {
        match self.room.room_type() {
            None => Default::default(),
            Some(room_type) => match room_type {
                RoomType::Space { .. } => "m.space".to_string(),
                RoomType::_Custom { .. } => "custom".to_string(),
                _ => Default::default(),
            },
        }
    }

    pub fn display_name(&self) -> String {
        self.room
            .name()
            .unwrap_or(self.id())
    }

    /// Has the room been tombstoned.
    pub fn is_tombstoned(&self) -> bool {
        self.room.is_tombstoned()
    }

    /// Get the `m.room.tombstone` content of this room if there is one.
    pub fn tombstone(&self) -> Box<RoomTombstoneEventContent> {
        Box::new(RoomTombstoneEventContent(self.room.tombstone_content()))
    }

    pub fn topic(&self) -> String {
        self.room.topic().unwrap_or_default()
    }

    pub fn num_unread_messages(&self) -> u64 {
        self.room.num_unread_messages()
    }

    pub fn num_unread_mentions(&self) -> u64 {
        self.room.num_unread_mentions()
    }

    /// Check whether the room is marked as favourite.
    ///
    /// A room is considered favourite if it has received the `m.favourite` tag.
    pub fn is_favourite(&self) -> bool {
        self.room.is_favourite()
    }

    /// Check whether the room is marked as low priority.
    ///
    /// A room is considered low priority if it has received the `m.lowpriority`
    /// tag.
    pub fn is_low_priority(&self) -> bool {
        self.room.is_low_priority()
    }
}
