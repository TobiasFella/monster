// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::prelude::{DateTime, Utc};
use matrix_sdk_ui::eyeball_im::VectorDiff;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::{
    media::MediaFormat,
    ruma::{RoomId, UserId},
    Client,
};
use matrix_sdk_ui::sync_service::SyncService;
use matrix_sdk_ui::timeline::{TimelineItemContent, TimelineItemKind, VirtualTimelineItem};
use std::mem::ManuallyDrop;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

struct Connection {
    rt: Runtime,
    client: Client,
}

/* Why Option<ManuallyDrop<...>>?
 * The drop function / destructor of room_list_service::Room expects to be called while in a tokio runtime,
 * which isn't what's happening, since the Rooms object is stored in C++. ManuallyDrop causes the destructor of the inner object to not be called by default,
 * which prevents this crash. To still clean it up, we implement drop for Rooms and call drop the object explicitely, after entering a runtime. This also requires the Option<...>, since it leaves a (very short) amount of time in which Rooms exists but the inner object has been destroyed already.
*/
struct Rooms(Option<ManuallyDrop<Arc<RwLock<Vec<matrix_sdk_ui::room_list_service::Room>>>>>);
struct RoomListRoom(matrix_sdk_ui::room_list_service::Room);

impl Rooms {
    fn room(&self, index: usize) -> Box<RoomListRoom> {
        Box::new(RoomListRoom(
            self.0.as_ref().unwrap().read().unwrap()[index].clone(),
        ))
    }

    fn count(&self) -> usize {
        self.0.as_ref().unwrap().read().unwrap().len()
    }
}

impl Drop for Rooms {
    fn drop(&mut self) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            drop(ManuallyDrop::into_inner(self.0.take().unwrap()));
        })
    }
}

impl RoomListRoom {
    fn id(&self) -> String {
        self.0.id().to_string()
    }
    fn display_name(&self) -> String {
        self.0
            .cached_display_name()
            .unwrap_or("No name available :(".to_string())
    }
}

/* There's two different types of RwLock in here!
 * Timeline.0 uses std::sync::RwLock, since this isn't async, which makes it easier to call from C++
 * Timeline.1 uses tokio's RwLock, which can be used in more complex async scenarios, but can only be acquired in an async function
 */
struct Timeline {
    queue: Arc<RwLock<Vec<VectorDiff<Arc<matrix_sdk_ui::timeline::TimelineItem>>>>>,
    timeline: Arc<tokio::sync::RwLock<matrix_sdk_ui::timeline::Timeline>>,
}

struct TimelineItem(Arc<matrix_sdk_ui::timeline::TimelineItem>);

struct VecDiff(VectorDiff<Arc<matrix_sdk_ui::timeline::TimelineItem>>);

impl VecDiff {
    fn op(&self) -> u8 {
        match self.0 {
            VectorDiff::Append { .. } => 0,
            VectorDiff::Clear => 1,
            VectorDiff::PushFront { .. } => 2,
            VectorDiff::PushBack { .. } => 3,
            VectorDiff::PopFront => 4,
            VectorDiff::PopBack => 5,
            VectorDiff::Insert { .. } => 6,
            VectorDiff::Set { .. } => 7,
            VectorDiff::Remove { .. } => 8,
            VectorDiff::Truncate { .. } => 9,
            VectorDiff::Reset { .. } => 10,
        }
    }

    fn index(&self) -> usize {
        match self.0 {
            VectorDiff::Insert { index, .. } => index,
            VectorDiff::Set { index, .. } => index,
            VectorDiff::Remove { index, .. } => index,
            VectorDiff::Truncate { length, .. } => length,
            _ => panic!(),
        }
    }

    fn item(&self) -> Box<TimelineItem> {
        match &self.0 {
            VectorDiff::Insert { value, .. } => Box::new(TimelineItem(value.clone())),
            VectorDiff::Set { value, .. } => Box::new(TimelineItem(value.clone())),
            VectorDiff::PushFront { value, .. } => Box::new(TimelineItem(value.clone())),
            VectorDiff::PushBack { value, .. } => Box::new(TimelineItem(value.clone())),
            _ => panic!(),
        }
    }

    fn items_vec(&self) -> Vec<TimelineItem> {
        match &self.0 {
            VectorDiff::Append { values, ..} => values.iter().map(|ti| TimelineItem(ti.clone())).collect(),
            VectorDiff::Reset { values, ..} => values.iter().map(|ti| TimelineItem(ti.clone())).collect(),
            _ => panic!()
        }
    }
}

impl Timeline {
    fn has_queued_item(&self) -> bool {
        !self.queue.read().unwrap().is_empty()
    }

    fn queue_next(&self) -> Box<VecDiff> {
        let mut write = self.queue.write().unwrap();
        let item = Box::new(VecDiff(write.first().unwrap().clone()));
        write.remove(0);
        item
    }
}

impl TimelineItem {
    fn id(&self) -> String {
        self.0
            .as_event()
            .map(|event| {
                event
                    .event_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    fn body(&self) -> String {
        match self.0.kind() {
            TimelineItemKind::Event(event) => match event.content() {
                TimelineItemContent::Message(message) => message.body().to_string(),
                event => format!("{:?}", event),
            },
            TimelineItemKind::Virtual(virt) => match virt {
                VirtualTimelineItem::DateDivider(millis) => format!("{}", millis.0),
                VirtualTimelineItem::ReadMarker => "Readmarker".to_string(),
            },
        }
    }

    fn timestamp(&self) -> String {
        match self.0.kind() {
            TimelineItemKind::Event(event) => {
                let dt: DateTime<Utc> = event.timestamp().to_system_time().unwrap().clone().into();
                format!("{}", dt.format("%+"))
            }
            _ => Default::default(),
        }
    }

    fn box_me(&self) -> Box<TimelineItem> {
        Box::new(TimelineItem(self.0.clone()))
    }
}

impl Connection {
    fn restore(secret: String) -> Box<Connection> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let session =
            serde_json::from_str::<MatrixSession>(std::str::from_utf8(secret.as_bytes()).unwrap())
                .unwrap();
        let matrix_id = session.meta.user_id.to_string();
        let client = rt.block_on(async {
            Client::builder()
                .server_name(UserId::parse(&session.meta.user_id).unwrap().server_name())
                .sqlite_store(
                    dirs::state_dir()
                        .unwrap()
                        .join("monster")
                        .join(session.meta.user_id.to_string()),
                    None, /* TODO: passphrase */
                )
                .build()
                .await
                .unwrap()
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            client_clone.restore_session(session).await.unwrap();
            ffi::shim_connected(matrix_id);
        });
        Box::new(Connection { rt, client })
    }

    fn timeline_paginate_back(&self, timeline: &Timeline) {
        let timeline = timeline.timeline.clone();
        self.rt.spawn(async move {
            timeline.write().await.paginate_backwards(20).await.unwrap();
        });
    }

    fn session(&self) -> String {
        use matrix_sdk::AuthSession;
        if let AuthSession::Matrix(session) = self.client.session().unwrap() {
            serde_json::to_string(&session).unwrap()
        } else {
            Default::default()
        }
    }

    fn init(matrix_id: String, password: String) -> Box<Connection> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let _ =
            std::fs::remove_dir_all(dirs::state_dir().unwrap().join("monster").join(&matrix_id));
        let client = rt.block_on(async {
            let user_id = UserId::parse(&matrix_id).unwrap();
            Client::builder()
                .server_name(&user_id.server_name())
                .sqlite_store(
                    dirs::state_dir().unwrap().join("monster").join(&matrix_id),
                    None, /* TODO: passphrase */
                )
                .build()
                .await
                .unwrap()
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            let user_id = UserId::parse(&matrix_id).unwrap();
            client_clone
                .matrix_auth()
                .login_username(user_id, &password)
                .send()
                .await
                .unwrap();
            ffi::shim_connected(matrix_id);
        });
        Box::new(Connection { rt, client })
    }

    fn timeline(&self, room_id: String) -> Box<Timeline> {
        let client = self.client.clone();
        let matrix_id = client
            .user_id()
            .map(|it| it.to_string())
            .unwrap_or("".to_string());
        let room_id = RoomId::parse(room_id).unwrap();
        let room = client.get_room(&room_id).unwrap();
        let (timeline, items, stream) = self.rt.block_on(async move {
            let timeline = matrix_sdk_ui::timeline::Timeline::builder(&room)
                .build()
                .await
                .unwrap();
            let (items, stream) = timeline.subscribe().await;
            (timeline, items, stream)
        });

        let timeline = Box::new(Timeline {
            queue: Arc::new(RwLock::new(vec![])),
            timeline: Arc::new(tokio::sync::RwLock::new(timeline)),
        });
        let queue = timeline.queue.clone();
        self.rt.spawn(async move {
            tokio::pin!(stream);

            let mxid = matrix_id.clone();

            queue.write().unwrap().push(VectorDiff::Append{values: items});
            ffi::shim_timeline_changed(mxid, room_id.to_string());

            loop {
                let matrix_id = matrix_id.clone();
                let room_id = room_id.to_string();
                let Some(entry) = stream.next().await else {
                    continue; //TODO or return?
                };

                queue.write().unwrap().push(entry);
                ffi::shim_timeline_changed(matrix_id, room_id);
            }
        });
        timeline
    }

    fn room_avatar(&self, room_id: String) {
        let client = self.rt.block_on(async { self.client.clone() });
        self.rt.spawn(async move {
            let room_id = RoomId::parse(room_id).unwrap();
            let data = client
                .get_room(&room_id)
                .unwrap()
                .avatar(MediaFormat::File)
                .await
                .unwrap()
                .unwrap_or("".into());
            ffi::shim_avatar_loaded(room_id.to_string(), data);
        });
    }

    fn device_id(&self) -> String {
        self.client.device_id().unwrap().to_string()
    }

    fn matrix_id(&self) -> String {
        self.client.user_id().unwrap().to_string()
    }

    fn slide(&self) -> Box<Rooms> {
        let client = self.client.clone();

        let rooms = Box::new(Rooms(Some(ManuallyDrop::new(Arc::new(RwLock::new(
            vec![],
        ))))));
        let rooms_clone = rooms.0.clone();
        self.rt.spawn(async move {
            let mut rooms = rooms_clone;
            let matrix_id = client
                .user_id()
                .map(|it| it.to_string())
                .unwrap_or("".to_string());
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
                            let from = rooms.as_ref().unwrap().read().unwrap().len();
                            let to = from + values.len() - 1;
                            {
                                let mut guard = rooms.as_mut().unwrap().write().unwrap();
                                for room in values {
                                    guard.push(room);
                                }
                            }
                            ffi::shim_rooms_changed(matrix_id.clone(), 0, from, to);
                        }
                        VectorDiff::Clear => {
                            let mut guard = rooms.as_ref().unwrap().write().unwrap();
                            let to = guard.len();
                            guard.clear();
                            ffi::shim_rooms_changed(matrix_id.clone(), 1, 0, to);
                        }
                        VectorDiff::PushFront { value } => {
                            rooms.as_mut().unwrap().write().unwrap().insert(0, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 2, 0, 0);
                        }
                        VectorDiff::PushBack { value } => {
                            let mut guard = rooms.as_mut().unwrap().write().unwrap();
                            let from = guard.len();
                            guard.push(value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 3, from, from);
                        }
                        VectorDiff::PopFront => {
                            rooms.as_mut().unwrap().write().unwrap().remove(0);
                            ffi::shim_rooms_changed(matrix_id.clone(), 4, 0, 0);
                        }
                        VectorDiff::PopBack => {
                            let mut guard = rooms.as_mut().unwrap().write().unwrap();
                            let from = guard.len() - 1;
                            guard.pop();
                            ffi::shim_rooms_changed(matrix_id.clone(), 5, from, from);
                        }
                        VectorDiff::Insert { index, value } => {
                            rooms
                                .as_mut()
                                .unwrap()
                                .write()
                                .unwrap()
                                .insert(index, value);
                            ffi::shim_rooms_changed(matrix_id.clone(), 6, index, index);
                        }
                        VectorDiff::Set { index, value } => {
                            rooms.as_mut().unwrap().write().unwrap()[index] = value;
                            ffi::shim_rooms_changed(matrix_id.clone(), 7, index, index);
                        }
                        VectorDiff::Remove { index } => {
                            rooms.as_mut().unwrap().write().unwrap().remove(index);
                            ffi::shim_rooms_changed(matrix_id.clone(), 8, index, index);
                        }
                        VectorDiff::Truncate { length } => {
                            let mut guard = rooms.as_mut().unwrap().write().unwrap();
                            let to = guard.len();
                            guard.truncate(length);
                            ffi::shim_rooms_changed(matrix_id.clone(), 9, length, to - 1);
                        }
                        VectorDiff::Reset { values } => {
                            rooms.as_mut().unwrap().write().unwrap().clear();
                            {
                                let mut guard = rooms.as_mut().unwrap().write().unwrap();
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

fn restore(secret: String) -> Box<Connection> {
    Connection::restore(secret)
}

// NOTE: When adding functions here, delete the entire build folder. There's probably something missing somewhere to make the header regenerate automatically
#[cxx::bridge]
mod ffi {
    #[namespace = "sdk"]
    extern "Rust" {
        type Connection;
        type RoomListRoom;
        type TimelineItem;
        type Rooms;
        type Timeline;
        type VecDiff;

        fn init(matrix_id: String, password: String) -> Box<Connection>;
        fn restore(secret: String) -> Box<Connection>;
        fn device_id(self: &Connection) -> String;
        fn matrix_id(self: &Connection) -> String;
        fn slide(self: &Connection) -> Box<Rooms>;
        fn room_avatar(self: &Connection, room_id: String);
        fn timeline(self: &Connection, room_id: String) -> Box<Timeline>;
        fn session(self: &Connection) -> String;
        fn timeline_paginate_back(self: &Connection, timeline: &Timeline);

        fn room(self: &Rooms, index: usize) -> Box<RoomListRoom>;
        fn count(self: &Rooms) -> usize;

        fn id(self: &RoomListRoom) -> String;
        fn display_name(self: &RoomListRoom) -> String;

        fn id(self: &TimelineItem) -> String;
        fn body(self: &TimelineItem) -> String;
        fn box_me(self: &TimelineItem) -> Box<TimelineItem>;
        fn timestamp(self: &TimelineItem) -> String;

        fn queue_next(self: &Timeline) -> Box<VecDiff>;
        fn has_queued_item(self: &Timeline) -> bool;

        fn op(self: &VecDiff) -> u8;
        fn index(self: &VecDiff) -> usize;
        fn item(self: &VecDiff) -> Box<TimelineItem>;
        fn items_vec(self: &VecDiff) -> Vec<TimelineItem>;

    }

    unsafe extern "C++" {
        include!("sdk/include/callbacks.h");

        fn shim_connected(matrix_id: String);
        fn shim_rooms_changed(matrix_id: String, op: u8, from: usize, to: usize);
        fn shim_timeline_changed(
            matrix_id: String,
            room_id: String,
        );
        fn shim_avatar_loaded(room_id: String, data: Vec<u8>);
    }
}
