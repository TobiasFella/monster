// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use matrix_sdk_ui::sync_service::SyncService;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use std::sync::Arc;
use tokio_stream::StreamExt;

use matrix_sdk::{
    media::MediaFormat, ruma::{RoomId, UserId}, Client
};


struct Connection {
    rt: Runtime,
    client: Arc<RwLock<Client>>,
    rooms: Arc<RwLock<Vec<matrix_sdk_ui::room_list_service::Room>>>,
    timeline_events: Arc<RwLock<Vec<Arc<matrix_sdk_ui::timeline::TimelineItem>>>>,
}

struct RoomListRoom(matrix_sdk_ui::room_list_service::Room);
struct TimelineItem(Arc<matrix_sdk_ui::timeline::TimelineItem>);

impl TimelineItem {
    fn id(&self) -> String {
        self.0.as_event().map(|event| event.event_id().map(|id| id.to_string()).unwrap_or("no_id".to_string())).unwrap_or("no id".to_string())
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

impl Connection {
    fn init(matrix_id: String, password: String) -> Box<Connection> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let client = rt.block_on(async {
            let user_id = UserId::parse(&matrix_id).unwrap();
            Arc::new(RwLock::new(Client::builder().server_name(&user_id.server_name()).build().await.unwrap()))
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            let user_id = UserId::parse(&matrix_id).unwrap();
            client_clone.read().await.matrix_auth().login_username(user_id, &password).send().await.unwrap();
            ffi::shim_connected(matrix_id);
        });
        Box::new(Connection {
            rt,
            client,
            rooms: Arc::new(RwLock::new(vec!())),
            timeline_events: Arc::new(RwLock::new(vec!())),
        })
    }

    fn timeline(&self, room_id: String) {
        let (client, timeline_events) = self.rt.block_on(async {
            (self.client.read().await.clone(), self.timeline_events.clone())
        });
        self.rt.spawn(async move {
            let matrix_id = client.user_id().map(|it| it.to_string()).unwrap_or("".to_string());
            let room_id = RoomId::parse(room_id).unwrap();
            let room = client.get_room(&room_id).unwrap();
            let builder = matrix_sdk_ui::timeline::Timeline::builder(&room);
            let timeline = builder.build().await.unwrap();
            let (_items, stream) = timeline.subscribe().await;

            tokio::pin!(stream);
            loop {
                let matrix_id = matrix_id.clone();
                let room_id = room_id.to_string();
                let Some(entry) = stream.next().await else {
                    continue; //TODO or return?
                };
                use matrix_sdk_ui::eyeball_im::VectorDiff;
                match entry {
                    VectorDiff::Append { values } => {
                        let from = timeline_events.read().await.len();
                        let to = from + values.len() - 1;
                        {
                            let mut guard = timeline_events.write().await;
                            for item in values {
                                guard.push(item);
                            }
                        }
                        ffi::shim_timeline_changed(matrix_id, room_id, 0, from, to);
                    }
                    VectorDiff::Clear => {
                        let mut guard = timeline_events.write().await;
                        let to = guard.len();
                        guard.clear();
                        ffi::shim_timeline_changed(matrix_id, room_id, 1, 0, to);
                    }
                    VectorDiff::PushFront { value } => {
                        timeline_events.write().await.insert(0, value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 2, 0, 0);
                    }
                    VectorDiff::PushBack { value } => {
                        let mut guard = timeline_events.write().await;
                        let from = guard.len();
                        guard.push(value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 3, from, from);
                    }
                    VectorDiff::PopFront => {
                        timeline_events.write().await.remove(0);
                        ffi::shim_timeline_changed(matrix_id, room_id, 4, 0, 0);
                    }
                    VectorDiff::PopBack => {
                        let mut guard = timeline_events.write().await;
                        let from = guard.len() - 1;
                        guard.pop();
                        ffi::shim_timeline_changed(matrix_id, room_id, 5, from, from);
                    }
                    VectorDiff::Insert { index, value } => {
                        timeline_events.write().await.insert(index, value);
                        ffi::shim_timeline_changed(matrix_id, room_id, 6, index, index);
                    }
                    VectorDiff::Set { index, value } => {
                        timeline_events.write().await[index] = value;
                        ffi::shim_timeline_changed(matrix_id, room_id, 7, index, index);
                    }
                    VectorDiff::Remove { index } => {
                        timeline_events.write().await.remove(index);
                        ffi::shim_timeline_changed(matrix_id, room_id, 8, index, index);
                    }
                    VectorDiff::Truncate { length } => {
                        let mut guard = timeline_events.write().await;
                        let to = guard.len();
                        guard.truncate(length);
                        ffi::shim_timeline_changed(matrix_id, room_id, 9, length, to - 1);
                    }
                    VectorDiff::Reset { values } => {
                        timeline_events.write().await.clear();
                        {
                            let mut guard = timeline_events.write().await;
                            for item in values {
                                guard.push(item);
                            }
                        }
                        ffi::shim_timeline_changed(matrix_id, room_id, 10, 0, 0);
                    }
                };
            }
        });
    }

    fn room_avatar(&self, room_id: String) {
        let client = self.rt.block_on(async {
            self.client.read().await.clone()
        });
        self.rt.spawn(async move {
            let room_id = RoomId::parse(room_id).unwrap();
            let data = client.get_room(&room_id).unwrap().avatar(MediaFormat::File).await.unwrap().unwrap();
            ffi::shim_avatar_loaded(room_id.to_string(), data);
        });
    }

    fn timeline_item(&self, index: usize) -> Box<TimelineItem> {
        self.rt.block_on(async {
            Box::new(TimelineItem(self.timeline_events.read().await[index].clone()))
        })
    }

    fn device_id(&self) -> String {
        self.rt.block_on(async {
            self.client.read().await.device_id().unwrap().to_string()
        })
    }

    fn room(&self, index: usize) -> Box<RoomListRoom> {
        self.rt.block_on(async {
            Box::new(RoomListRoom(self.rooms.read().await[index].clone()))
        })
    }

    fn room_event_count(&self, _room_id: String) -> usize {
        self.rt.block_on(async {
            self.timeline_events.read().await.len()
        })
    }

    fn rooms_count(&self) -> usize {
        self.rt.block_on(async {
            self.rooms.read().await.len()
        })
    }

    fn slide(&self) {
        let (client, rooms) = self.rt.block_on(async {
            (self.client.read().await.clone(), self.rooms.clone())
        });
        self.rt.spawn(async move {
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
                            let from = rooms.read().await.len();
                            let to = from + values.len() - 1;
                            {
                                let mut guard = rooms.write().await;
                                for room in values {
                                    guard.push(room);
                                }
                            }
                            ffi::shim_rooms_changed(matrix_id.clone(), 0, from, to);
                        }
                        VectorDiff::Clear => {
                            let mut guard = rooms.write().await;
                            let to = guard.len();
                            guard.clear();
                            ffi::shim_rooms_changed(matrix_id.clone(), 1, 0, to);
                        }
                        VectorDiff::PushFront { value } => {
                            rooms.write().await.insert(0, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 2, 0, 0);
                        }
                        VectorDiff::PushBack { value } => {
                            let mut guard = rooms.write().await;
                            let from = guard.len();
                            guard.push(value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 3, from, from);
                        }
                        VectorDiff::PopFront => {
                            rooms.write().await.remove(0);
                            ffi::shim_rooms_changed(matrix_id.clone(), 4, 0, 0);
                        }
                        VectorDiff::PopBack => {
                            let mut guard = rooms.write().await;
                            let from = guard.len() - 1;
                            guard.pop();
                            ffi::shim_rooms_changed(matrix_id.clone(), 5, from, from);
                        }
                        VectorDiff::Insert { index, value } => {
                            rooms.write().await.insert(index, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 6, index, index);
                        }
                        VectorDiff::Set { index, value } => {
                            rooms.write().await[index] = value;
                            ffi::shim_rooms_changed(matrix_id.clone(), 7, index, index);
                        }
                        VectorDiff::Remove { index } => {
                            rooms.write().await.remove(index);
                            ffi::shim_rooms_changed(matrix_id.clone(), 8, index, index);
                        }
                        VectorDiff::Truncate { length } => {
                            let mut guard = rooms.write().await;
                            let to = guard.len();
                            guard.truncate(length);
                            ffi::shim_rooms_changed(matrix_id.clone(), 9, length, to - 1);
                        }
                        VectorDiff::Reset { values } => {
                            rooms.write().await.clear();
                            {
                                let mut guard = rooms.write().await;
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
        fn init(matrix_id: String, password: String) -> Box<Connection>;
        fn device_id(self: &Connection) -> String;
        fn slide(self: &Connection);
        fn room(self: &Connection, index: usize) -> Box<RoomListRoom>;
        fn rooms_count(self: &Connection) -> usize;
        fn room_avatar(self: &Connection, room_id: String);
        fn timeline(self: &Connection, room_id: String);
        fn room_event_count(self: &Connection, room_id: String) -> usize;
        fn timeline_item(self: &Connection, index: usize) -> Box<TimelineItem>;

        fn id(self: &RoomListRoom) -> String;
        fn display_name(self: &RoomListRoom) -> String;

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
