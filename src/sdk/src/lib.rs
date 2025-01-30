// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use matrix_sdk_ui::sync_service::SyncService;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use std::sync::Arc;
use tokio_stream::StreamExt;

use matrix_sdk::{
    Client,
    ruma::UserId,
};


struct Connection {
    rt: Runtime,
    client: Arc<RwLock<Client>>,
    rooms: Arc<RwLock<Vec<matrix_sdk_ui::room_list_service::Room>>>,
}

struct RoomListRoom(matrix_sdk_ui::room_list_service::Room);

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
            ffi::shim_connected();
        });
        Box::new(Connection {
            rt,
            client,
            rooms: Arc::new(RwLock::new(vec!())),
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
            let sync_service = SyncService::builder(client).build().await.unwrap();
            let service = sync_service.room_list_service();
            sync_service.start().await;
            let room_list = service.all_rooms().await.unwrap();
            let (stream, controller) = room_list.entries_with_dynamic_adapters(10000);
            use tokio::pin;
            pin!(stream);
            controller.set_filter(Box::new(|_| true));
            loop {
                for entry in stream.next().await.unwrap() {
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
                            ffi::shim_rooms_changed(0, from, to);
                        }
                        VectorDiff::Clear => {
                            let mut guard = rooms.write().await;
                            let to = guard.len();
                            guard.clear();
                            ffi::shim_rooms_changed(1, 0, to);
                        }
                        VectorDiff::PushFront { value } => {
                            rooms.write().await.insert(0, value);
                            ffi::shim_rooms_changed(2, 0, 0);
                        }
                        VectorDiff::PushBack { value } => {
                            let mut guard = rooms.write().await;
                            let from = guard.len();
                            guard.push(value);
                            ffi::shim_rooms_changed(3, from, from);
                        }
                        VectorDiff::PopFront => {
                            rooms.write().await.remove(0);
                            ffi::shim_rooms_changed(4, 0, 0);
                        }
                        VectorDiff::PopBack => {
                            let mut guard = rooms.write().await;
                            let from = guard.len() - 1;
                            guard.pop();
                            ffi::shim_rooms_changed(5, from, from);
                        }
                        VectorDiff::Insert { index, value } => {
                            rooms.write().await.insert(index, value);
                            ffi::shim_rooms_changed(6, index, index);
                        }
                        VectorDiff::Set { index, value } => {
                            rooms.write().await[index] = value;
                            ffi::shim_rooms_changed(7, index, index);
                        }
                        VectorDiff::Remove { index } => {
                            rooms.write().await.remove(index);
                            ffi::shim_rooms_changed(8, index, index);
                        }
                        VectorDiff::Truncate { length } => {
                            let mut guard = rooms.write().await;
                            let to = guard.len();
                            guard.truncate(length);
                            ffi::shim_rooms_changed(9, length, to - 1);
                        }
                        VectorDiff::Reset { values } => {
                            rooms.write().await.clear();
                            {
                                let mut guard = rooms.write().await;
                                for room in values {
                                    guard.push(room);
                                }
                            }
                            ffi::shim_rooms_changed(10, 0, 0);
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
        fn init(matrix_id: String, password: String) -> Box<Connection>;
        fn device_id(self: &Connection) -> String;
        fn slide(self: &Connection);
        fn room(self: &Connection, index: usize) -> Box<RoomListRoom>;
        fn rooms_count(self: &Connection) -> usize;

        fn id(self: &RoomListRoom) -> String;
        fn display_name(self: &RoomListRoom) -> String;
    }

    unsafe extern "C++" {
        include!("sdk/include/callbacks.h");

        fn shim_connected();
        fn shim_rooms_changed(op: u8, from: usize, to: usize);
    }
}
