// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::prelude::{DateTime, Utc};
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::ruma::api::client::room::Visibility;
use matrix_sdk::ruma::events::room::message::{
    MessageType, RoomMessageEventContent, TextMessageEventContent,
};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::{
    media::MediaFormat,
    ruma::{RoomId, UserId},
    Client,
};
use matrix_sdk_ui::eyeball_im::VectorDiff;
use matrix_sdk_ui::sync_service::SyncService;
use matrix_sdk_ui::timeline::{TimelineItemContent, TimelineItemKind, VirtualTimelineItem};
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

struct Connection {
    rt: Runtime,
    client: Client,
}

struct Rooms {
    queue: Arc<RwLock<Vec<VectorDiff<matrix_sdk_ui::room_list_service::Room>>>>,
}

impl Room {
    fn display_name(&self) -> String {
        self.room.name().unwrap_or("This room does not have a name".to_string())
    }

    fn id(&self) -> String {
        self.room.room_id().to_string()
    }
}

struct Room {
    room: matrix_sdk::room::Room,
}

struct RoomListRoom(matrix_sdk_ui::room_list_service::Room);

struct RoomListVecDiff(VectorDiff<matrix_sdk_ui::room_list_service::Room>);

impl Rooms {
    fn has_queued_item(&self) -> bool {
        !self.queue.read().unwrap().is_empty()
    }

    fn queue_next(&self) -> Box<RoomListVecDiff> {
        let mut write = self.queue.write().unwrap();
        let item = Box::new(RoomListVecDiff(write.first().unwrap().clone()));
        write.remove(0);
        item
    }
}

impl RoomListVecDiff {
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

    fn item(&self) -> Box<RoomListRoom> {
        match &self.0 {
            VectorDiff::Insert { value, .. } => Box::new(RoomListRoom(value.clone())),
            VectorDiff::Set { value, .. } => Box::new(RoomListRoom(value.clone())),
            VectorDiff::PushFront { value, .. } => Box::new(RoomListRoom(value.clone())),
            VectorDiff::PushBack { value, .. } => Box::new(RoomListRoom(value.clone())),
            _ => panic!(),
        }
    }

    fn items_vec(&self) -> Vec<RoomListRoom> {
        match &self.0 {
            VectorDiff::Append { values, .. } => {
                values.iter().map(|ti| RoomListRoom(ti.clone())).collect()
            }
            VectorDiff::Reset { values, .. } => {
                values.iter().map(|ti| RoomListRoom(ti.clone())).collect()
            }
            _ => panic!(),
        }
    }
}

// impl Drop for Rooms {
//     fn drop(&mut self) {
//         tokio::runtime::Runtime::new().unwrap().block_on(async {
//             drop(ManuallyDrop::into_inner(self.0.take().unwrap()));
//         })
//     }
// }

impl RoomListRoom {
    fn id(&self) -> String {
        self.0.id().to_string()
    }
    fn display_name(&self) -> String {
        self.0
            .cached_display_name()
            .unwrap_or("No name available :(".to_string())
    }

    fn box_me(&self) -> Box<RoomListRoom> {
        Box::new(RoomListRoom(self.0.clone()))
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
            VectorDiff::Append { values, .. } => {
                values.iter().map(|ti| TimelineItem(ti.clone())).collect()
            }
            VectorDiff::Reset { values, .. } => {
                values.iter().map(|ti| TimelineItem(ti.clone())).collect()
            }
            _ => panic!(),
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

    fn send_message(&self, connection: &Connection, message: String) {
        let timeline = self.timeline.clone();
        connection.rt.spawn(async move {
            let content = RoomMessageEventContent::new(MessageType::Text(
                TextMessageEventContent::plain(message),
            ));
            timeline
                .write()
                .await
                .send(AnyMessageLikeEventContent::RoomMessage(content))
                .await
                .unwrap();
        });
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

            queue
                .write()
                .unwrap()
                .push(VectorDiff::Append { values: items });
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

        let rooms = Box::new(Rooms {
            queue: Arc::new(RwLock::new(vec![])),
        });
        let rooms_clone = rooms.queue.clone();
        self.rt.spawn(async move {
            let rooms = rooms_clone;
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
                    rooms.write().unwrap().push(entry);
                    ffi::shim_rooms_changed(m.clone());
                }
            }
        });
        rooms
    }

    fn logout(&self) {
        let client = self.client.clone();
        self.rt.spawn(async move {
            let result = client.matrix_auth().logout().await;
            use http::status::StatusCode;
            use matrix_sdk::ruma::api::client::error::{ErrorBody, ErrorKind};
            use matrix_sdk::ruma::api::client::Error;
            use matrix_sdk::ruma::api::error::FromHttpResponseError::Server;
            use matrix_sdk::HttpError::Api;
            use matrix_sdk::RumaApiError::ClientApi;
            match result {
                Err(Api(Server(ClientApi(Error {
                    status_code: StatusCode::UNAUTHORIZED,
                    body:
                        ErrorBody::Standard {
                            kind: ErrorKind::UnknownToken { .. },
                            ..
                        },
                    ..
                }))))
                | Ok(..) => {
                    ffi::shim_logged_out(client.user_id().unwrap().to_string());
                }
                x => eprintln!("Error logging out: {:?}", x),
            }
        });
    }

    fn create_room(&self, room_create_options: &RoomCreateOptions) {
        let client = self.client.clone();
        let options = room_create_options.0.clone();
        self.rt.spawn(async move {
            client.create_room(options).await.unwrap();
        });
    }

    fn room(&self, id: String) -> Box<Room> {
        let room_id = RoomId::parse(id).unwrap();
        Box::new(Room { room: self.client.get_room(&room_id).unwrap() })
    }
}

#[derive(Clone)]
struct RoomCreateOptions(matrix_sdk::ruma::api::client::room::create_room::v3::Request);

impl RoomCreateOptions {
    fn set_invite(&mut self, users: Vec<String>) {
        self.0.invite = users.iter().map(|it| UserId::parse(&it).unwrap()).collect()
    }

    fn set_name(&mut self, name: String) {
        self.0.name = Some(name);
    }

    fn set_room_alias(&mut self, alias: String) {
        self.0.room_alias_name = Some(alias);
    }

    fn set_topic(&mut self, topic: String) {
        self.0.topic = Some(topic);
    }

    fn set_visibility_public(&mut self, visibility_public: bool) {
        self.0.visibility = if visibility_public {
            Visibility::Public
        } else {
            Visibility::Private
        };
    }
    // creation_content: Option<Raw<CreationContent>>,
    // initial_state: Vec<Raw<AnyInitialStateEvent>>,
    // power_level_content_override: Option<Raw<RoomPowerLevelsEventContent>>,
    // preset: Option<RoomPreset>,
    // room_alias_name: Option<String>,
    // room_version: Option<RoomVersionId>,
}

fn room_create_options_new() -> Box<RoomCreateOptions> {
    Box::new(RoomCreateOptions(
        matrix_sdk::ruma::api::client::room::create_room::v3::Request::new(),
    ))
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
        type RoomListVecDiff;
        type RoomCreateOptions;
        type Room;

        fn init(matrix_id: String, password: String) -> Box<Connection>;
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

        fn id(self: &RoomListRoom) -> String;
        fn display_name(self: &RoomListRoom) -> String;
        fn box_me(self: &RoomListRoom) -> Box<RoomListRoom>;

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
        fn item(self: &RoomListVecDiff) -> Box<RoomListRoom>;
        fn items_vec(self: &RoomListVecDiff) -> Vec<RoomListRoom>;

        fn room_create_options_new() -> Box<RoomCreateOptions>;
        fn set_invite(self: &mut RoomCreateOptions, users: Vec<String>);
        fn set_name(self: &mut RoomCreateOptions, name: String);
        fn set_room_alias(self: &mut RoomCreateOptions, alias: String);
        fn set_topic(self: &mut RoomCreateOptions, topic: String);
        fn set_visibility_public(self: &mut RoomCreateOptions, visibility_public: bool);

        fn display_name(self: &Room) -> String;
        fn id(self: &Room) -> String;
    }

    unsafe extern "C++" {
        include!("sdk/include/callbacks.h");

        fn shim_connected(matrix_id: String);
        fn shim_rooms_changed(matrix_id: String);
        fn shim_timeline_changed(matrix_id: String, room_id: String);
        fn shim_avatar_loaded(room_id: String, data: Vec<u8>);
        fn shim_logged_out(matrix_id: String);
    }
}
