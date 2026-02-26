// SPDX-FileCopyrightText: 2026 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

use crate::*;

// NOTE: When adding functions here, delete the entire build folder. There's probably something missing somewhere to make the header regenerate automatically
#[cxx::bridge]
pub mod ffi {
    #[namespace = "sdk"]
    extern "Rust" {
        type RoomTombstoneEventContent;
        type Connection;
        type RoomListItem;
        type TimelineItem;
        type Rooms;
        type Timeline;
        type VecDiff;
        type RoomListVecDiff;
        type RoomCreateOptions;
        type Room;

        pub fn body(self: &RoomTombstoneEventContent) -> String;
        pub fn replacement_room(self: &RoomTombstoneEventContent) -> String;

        fn init(matrix_id: String, password: String) -> Box<Connection>;
        fn init_oidc(server_name: String) -> Box<Connection>;
        fn restore(secret: String) -> Box<Connection>;
        fn device_id(self: &Connection) -> String;
        fn matrix_id(self: &Connection) -> String;
        fn slide(self: &Connection) -> Box<Rooms>;
        fn room_avatar(self: &Connection, room_id: String);
        fn timeline(self: &Connection, room_id: String) -> Box<Timeline>;
        fn session(self: &Connection) -> String;
        fn timeline_paginate_back(self: &Connection, timeline: &Timeline);
        fn logout(self: &Connection);
        fn create_room(self: &Connection, room_create_options: &RoomCreateOptions);
        fn room(self: &Connection, id: String) -> Box<Room>;
        fn is_known_room(self: &Connection, id: String) -> bool;

        fn set_display_name(self: &Connection, display_name: String) -> String;

        fn id(self: &TimelineItem) -> String;
        fn body(self: &TimelineItem) -> String;
        fn box_me(self: &TimelineItem) -> Box<TimelineItem>;
        fn timestamp(self: &TimelineItem) -> String;

        fn queue_next(self: &Timeline) -> Box<VecDiff>;
        fn has_queued_item(self: &Timeline) -> bool;
        fn send_message(self: &Timeline, connection: &Connection, message: String);

        fn queue_next(self: &Rooms) -> Box<RoomListVecDiff>;
        fn has_queued_item(self: &Rooms) -> bool;

        fn op(self: &VecDiff) -> u8;
        fn index(self: &VecDiff) -> usize;
        fn item(self: &VecDiff) -> Box<TimelineItem>;
        fn items_vec(self: &VecDiff) -> Vec<TimelineItem>;

        fn op(self: &RoomListVecDiff) -> u8;
        fn index(self: &RoomListVecDiff) -> usize;
        fn item(self: &RoomListVecDiff) -> Box<RoomListItem>;
        fn items_vec(self: &RoomListVecDiff) -> Vec<RoomListItem>;

        fn room_create_options_new() -> Box<RoomCreateOptions>;
        fn set_invite(self: &mut RoomCreateOptions, users: Vec<String>);
        fn set_name(self: &mut RoomCreateOptions, name: String);
        fn set_room_alias(self: &mut RoomCreateOptions, alias: String);
        fn set_topic(self: &mut RoomCreateOptions, topic: String);
        fn set_visibility_public(self: &mut RoomCreateOptions, visibility_public: bool);

        fn id(self: &Room) -> String;
        fn state(self: &Room) -> u8;
        fn is_space(self: &Room) -> bool;
        fn room_type(self: &Room) -> String;
        fn display_name(self: &Room) -> String;
        fn is_tombstoned(self: &Room) -> bool;
        fn tombstone(self: &Room) -> Box<RoomTombstoneEventContent>;
        fn topic(self: &Room) -> String;
        fn num_unread_messages(self: &Room) -> u64;
        fn num_unread_mentions(self: &Room) -> u64;
        fn is_favourite(self: &Room) -> bool;
        fn is_low_priority(self: &Room) -> bool;

        fn id(self: &RoomListItem) -> String;
        fn state(self: &RoomListItem) -> u8;
        fn is_space(self: &RoomListItem) -> bool;
        fn room_type(self: &RoomListItem) -> String;
        fn display_name(self: &RoomListItem) -> String;
        fn is_tombstoned(self: &RoomListItem) -> bool;
        fn tombstone(self: &RoomListItem) -> Box<RoomTombstoneEventContent>;
        fn topic(self: &RoomListItem) -> String;
        fn num_unread_messages(self: &RoomListItem) -> u64;
        fn num_unread_mentions(self: &RoomListItem) -> u64;
        fn canonical_alias(self: &RoomListItem) -> String;
        fn is_favourite(self: &RoomListItem) -> bool;
        fn is_low_priority(self: &RoomListItem) -> bool;
        fn box_me(self: &RoomListItem) -> Box<RoomListItem>;
    }

    unsafe extern "C++" {
        include!("sdk/include/callbacks.h");

        pub fn shim_connected(matrix_id: String);
        pub fn shim_rooms_changed(matrix_id: String);
        pub fn shim_timeline_changed(matrix_id: String, room_id: String);
        pub fn shim_avatar_loaded(room_id: String, data: Vec<u8>);
        pub fn shim_logged_out(matrix_id: String);

        pub fn shim_oidc_login_url_available(server_name: String, url: String);

        pub fn task_done(token: String);
    }
}