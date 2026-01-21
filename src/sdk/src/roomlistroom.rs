// SPDX-FileCopyrightText: 2025 James Graham <james.h.graham@protonmail.com>
// SPDX-License-Identifier: LGPL-2.0-or-later

use matrix_sdk::RoomState;
use matrix_sdk::ruma::room::RoomType;

use crate::tombstone::RoomTombstoneEventContent;

pub struct RoomListRoom(pub matrix_sdk_ui::room_list_service::Room);

impl RoomListRoom {
    pub fn id(&self) -> String {
        self.0.id().to_string()
    }

    /// Get the state of the room.
    pub fn state(&self) -> u8 {
        match self.0.state() {
            RoomState::Joined { .. } => 0,
            RoomState::Left { .. } => 1,
            RoomState::Invited { .. } => 2,
            RoomState::Knocked { .. } => 3,
            RoomState::Banned { .. } => 4,
        }
    }

    /// Whether this room's [`RoomType`] is `m.space`.
    pub fn is_space(&self) -> bool {
        self.0.is_space()
    }

    /// Returns the room's type as defined in its creation event
    /// (`m.room.create`).
    pub fn room_type(&self) -> String {
        match self.0.room_type() {
            None => Default::default(),
            Some(room_type) => match room_type {
                RoomType::Space { .. } => "m.space".to_string(),
                RoomType::_Custom { .. } => "custom".to_string(),
                _ => Default::default(),
            },
        }
    }

    pub fn display_name(&self) -> String {
        self.0
            .cached_display_name()
            .unwrap_or(self.id())
    }

    /// Has the room been tombstoned.
    pub fn is_tombstoned(&self) -> bool {
        self.0.is_tombstoned()
    }

    /// Get the `m.room.tombstone` content of this room if there is one.
    pub fn tombstone(&self) -> Box<RoomTombstoneEventContent> {
        match self.0.tombstone() {
            None => Box::new(RoomTombstoneEventContent::empty()),
            Some(inner_content) => Box::new(RoomTombstoneEventContent(Some(inner_content))),
        }
    }

    pub fn topic(&self) -> String {
        self.0.topic().unwrap_or_default()
    }

    pub fn num_unread_messages(&self) -> u64 {
        self.0.num_unread_messages()
    }

    pub fn num_unread_mentions(&self) -> u64 {
        self.0.num_unread_mentions()
    }

    /// Get the canonical alias of this room.
    pub fn canonical_alias(&self) -> String {
        match self.0.canonical_alias() {
            None => Default::default(),
            Some(alias) => alias.to_string(),
        }
    }

    /// Check whether the room is marked as favourite.
    ///
    /// A room is considered favourite if it has received the `m.favourite` tag.
    pub fn is_favourite(&self) -> bool {
        self.0.is_favourite()
    }

    /// Check whether the room is marked as low priority.
    ///
    /// A room is considered low priority if it has received the `m.lowpriority`
    /// tag.
    pub fn is_low_priority(&self) -> bool {
        self.0.is_low_priority()
    }

    pub fn box_me(&self) -> Box<RoomListRoom> {
        Box::new(RoomListRoom(self.0.clone()))
    }
}
