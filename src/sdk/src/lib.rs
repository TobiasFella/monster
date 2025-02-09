// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use matrix_sdk_ui::sync_service::SyncService;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use std::sync::{Arc, RwLock};

use matrix_sdk::{
    media::MediaFormat, ruma::{RoomId, UserId}, Client
};

struct Connection {
    rt: Runtime,
    client: Client,
}

struct Rooms(Arc<RwLock<Vec<matrix_sdk_ui::room_list_service::Room>>>);
struct RoomListRoom(matrix_sdk_ui::room_list_service::Room);

impl Rooms {
    fn room(&self, index: usize) -> Box<RoomListRoom> {
        Box::new(RoomListRoom(self.0.read().unwrap()[index].clone()))
    }

    fn count(&self) -> usize {
        self.0.read().unwrap().len()
    }
}

impl RoomListRoom {
    fn id(&self) -> String {
        self.0.id().to_string()
    }
    fn display_name(&self) -> String {
        self.0.cached_display_name().unwrap_or("No name available :(".to_string())
    }
}

struct Timeline(Arc<RwLock<Vec<Arc<matrix_sdk_ui::timeline::TimelineItem>>>>);
struct TimelineItem(Arc<matrix_sdk_ui::timeline::TimelineItem>);

impl Timeline {
    fn count(&self) -> usize {
        self.0.read().unwrap().len()
    }

    fn timeline_item(&self, index: usize) -> Box<TimelineItem> {
        Box::new(TimelineItem(self.0.read().unwrap()[index].clone()))
    }
}

impl TimelineItem {
    fn id(&self) -> String {
        self.0.as_event().map(|event| event.event_id().map(|id| id.to_string()).unwrap_or("no_id".to_string())).unwrap_or("no id".to_string())
    }
}

impl Connection {
    fn init(matrix_id: String, password: String) -> Box<Connection> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let client = rt.block_on(async {
            let user_id = UserId::parse(&matrix_id).unwrap();
            Client::builder().server_name(&user_id.server_name()).build().await.unwrap()
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            let user_id = UserId::parse(&matrix_id).unwrap();
            client_clone.matrix_auth().login_username(user_id, &password).send().await.unwrap();
            ffi::shim_connected(matrix_id);
        });
        Box::new(Connection {
            rt,
            client,
        })
    }

    fn timeline(&self, room_id: String) -> Box<Timeline> {
        let client = self.client.clone();

        let timeline = Box::new(Timeline(Arc::new(RwLock::new(vec!()))));
        let timeline_clone = timeline.0.clone();
        self.rt.spawn(async move {
            let timeline = timeline_clone;
            let matrix_id = client.user_id().map(|it| it.to_string()).unwrap_or("".to_string());
            let room_id = RoomId::parse(room_id).unwrap();
            let room = client.get_room(&room_id).unwrap();
            let (items, stream) = matrix_sdk_ui::timeline::Timeline::builder(&room).build().await.unwrap().subscribe().await;
            tokio::pin!(stream);

            {
                let mut write = timeline.write().unwrap();
                for item in items {
                    write.push(item);
                }
            }

            loop {
                let matrix_id = matrix_id.clone();
                let room_id = room_id.to_string();
                let Some(entry) = stream.next().await else {
                    continue; //TODO or return?
                };
                use matrix_sdk_ui::eyeball_im::VectorDiff;
                match entry {
                    VectorDiff::Append { values } => {
                        let from = timeline.read().unwrap().len();
                        let to = from + values.len() - 1;
                        {
                            let mut guard = timeline.write().unwrap();
                            for item in values {
                                guard.push(item);
                            }
                        }
                        ffi::shim_timeline_changed(matrix_id, room_id, 0, from, to);
                    }
                    VectorDiff::Clear => {
                        let mut guard = timeline.write().unwrap();
                        let to = guard.len();
                        guard.clear();
                        ffi::shim_timeline_changed(matrix_id, room_id, 1, 0, to);
                    }
                    VectorDiff::PushFront { value } => {
                        timeline.write().unwrap().insert(0, value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 2, 0, 0);
                    }
                    VectorDiff::PushBack { value } => {
                        let mut guard = timeline.write().unwrap();
                        let from = guard.len();
                        guard.push(value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 3, from, from);
                    }
                    VectorDiff::PopFront => {
                        timeline.write().unwrap().remove(0);
                        ffi::shim_timeline_changed(matrix_id, room_id, 4, 0, 0);
                    }
                    VectorDiff::PopBack => {
                        let mut guard = timeline.write().unwrap();
                        let from = guard.len() - 1;
                        guard.pop();
                        ffi::shim_timeline_changed(matrix_id, room_id, 5, from, from);
                    }
                    VectorDiff::Insert { index, value } => {
                        timeline.write().unwrap().insert(index, value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 6, index, index);
                    }
                    VectorDiff::Set { index, value } => {
                        timeline.write().unwrap()[index] = value;
                        ffi::shim_timeline_changed(matrix_id, room_id, 7, index, index);
                    }
                    VectorDiff::Remove { index } => {
                        timeline.write().unwrap().remove(index);
                        ffi::shim_timeline_changed(matrix_id, room_id, 8, index, index);
                    }
                    VectorDiff::Truncate { length } => {
                        let mut guard = timeline.write().unwrap();
                        let to = guard.len();
                        guard.truncate(length);
                        ffi::shim_timeline_changed(matrix_id, room_id, 9, length, to - 1);
                    }
                    VectorDiff::Reset { values } => {
                        timeline.write().unwrap().clear();
                        {
                            let mut guard = timeline.write().unwrap();
                            for item in values {
                                guard.push(item);
                            }
                        }
                        ffi::shim_timeline_changed(matrix_id, room_id, 10, 0, 0);
                    }
                };
            }
        });
        timeline
    }

    fn room_avatar(&self, room_id: String) {
        let client = self.rt.block_on(async {
            self.client.clone()
        });
        self.rt.spawn(async move {
            let room_id = RoomId::parse(room_id).unwrap();
            let data = client.get_room(&room_id).unwrap().avatar(MediaFormat::File).await.unwrap().unwrap_or("".into());
            ffi::shim_avatar_loaded(room_id.to_string(), data);
        });
    }

    fn device_id(&self) -> String {
        self.rt.block_on(async {
            self.client.device_id().unwrap().to_string()
        })
    }

    fn slide(&self) -> Box<Rooms> {
        let client = self.client.clone();

        let rooms = Box::new(Rooms(Arc::new(RwLock::new(vec!()))));
        let rooms_clone = rooms.0.clone();
        self.rt.spawn(async move {
            let rooms = rooms_clone;
            let matrix_id = client.user_id().map(|it| it.to_string()).unwrap_or("".to_string());
            let sync_service = SyncService::builder(client).build().await.unwrap();
            let service = sync_service.room_list_service();
            sync_service.start().await;
            let room_list = service.all_rooms().await.unwrap();
            let (stream, controller) = room_list.entries_with_dynamic_adapters(10000);
            use tokio::pin;
            pin!(stream);
            controller.set_filter(Box::new(|_| true));
            loop {
                let m = matrix_id.clone();
                for entry in stream.next().await.unwrap() {
                    let matrix_id = m.clone();
                    use matrix_sdk_ui::eyeball_im::VectorDiff;
                    match entry {
                        VectorDiff::Append { values } => {
                            let from = rooms.read().unwrap().len();
                            let to = from + values.len() - 1;
                            {
                                let mut guard = rooms.write().unwrap();
                                for room in values {
                                    guard.push(room);
                                }
                            }
                            ffi::shim_rooms_changed(matrix_id.clone(), 0, from, to);
                        }
                        VectorDiff::Clear => {
                            let mut guard = rooms.write().unwrap();
                            let to = guard.len();
                            guard.clear();
                            ffi::shim_rooms_changed(matrix_id.clone(), 1, 0, to);
                        }
                        VectorDiff::PushFront { value } => {
                            rooms.write().unwrap().insert(0, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 2, 0, 0);
                        }
                        VectorDiff::PushBack { value } => {
                            let mut guard = rooms.write().unwrap();
                            let from = guard.len();
                            guard.push(value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 3, from, from);
                        }
                        VectorDiff::PopFront => {
                            rooms.write().unwrap().remove(0);
                            ffi::shim_rooms_changed(matrix_id.clone(), 4, 0, 0);
                        }
                        VectorDiff::PopBack => {
                            let mut guard = rooms.write().unwrap();
                            let from = guard.len() - 1;
                            guard.pop();
                            ffi::shim_rooms_changed(matrix_id.clone(), 5, from, from);
                        }
                        VectorDiff::Insert { index, value } => {
                            rooms.write().unwrap().insert(index, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 6, index, index);
                        }
                        VectorDiff::Set { index, value } => {
                            rooms.write().unwrap()[index] = value;
                            ffi::shim_rooms_changed(matrix_id.clone(), 7, index, index);
                        }
                        VectorDiff::Remove { index } => {
                            rooms.write().unwrap().remove(index);
                            ffi::shim_rooms_changed(matrix_id.clone(), 8, index, index);
                        }
                        VectorDiff::Truncate { length } => {
                            let mut guard = rooms.write().unwrap();
                            let to = guard.len();
                            guard.truncate(length);
                            ffi::shim_rooms_changed(matrix_id.clone(), 9, length, to - 1);
                        }
                        VectorDiff::Reset { values } => {
                            rooms.write().unwrap().clear();
                            {
                                let mut guard = rooms.write().unwrap();
                                for room in values {
                                    guard.push(room);
                                }
                            }
                            ffi::shim_rooms_changed(matrix_id.clone(), 10, 0, 0);
                        }
                    };
                }
            }
        });
        rooms
    }
}

fn init(matrix_id: String, password: String) -> Box<Connection> {
    Connection::init(matrix_id, password)
}

#[cxx::bridge]
mod ffi {
    #[namespace = "sdk"]
    extern "Rust" {
        type Connection;
        type RoomListRoom;
        type TimelineItem;
        type Rooms;
        type Timeline;

        fn init(matrix_id: String, password: String) -> Box<Connection>;
        fn device_id(self: &Connection) -> String;
        fn slide(self: &Connection) -> Box<Rooms>;
        fn room_avatar(self: &Connection, room_id: String);
        fn timeline(self: &Connection, room_id: String) -> Box<Timeline>;

        fn room(self: &Rooms, index: usize) -> Box<RoomListRoom>;
        fn count(self: &Rooms) -> usize;

        fn id(self: &RoomListRoom) -> String;
        fn display_name(self: &RoomListRoom) -> String;

        fn count(self: &Timeline) -> usize;
        fn timeline_item(self: &Timeline, index: usize) -> Box<TimelineItem>;
        fn id(self: &TimelineItem) -> String;
    }

    unsafe extern "C++" {
        include!("sdk/include/callbacks.h");

        fn shim_connected(matrix_id: String);
        fn shim_rooms_changed(matrix_id: String, op: u8, from: usize, to: usize);
        fn shim_timeline_changed(matrix_id: String, room_id: String, op: u8, from: usize, to: usize);
        fn shim_avatar_loaded(room_id: String, data: Vec<u8>);
    }
}
