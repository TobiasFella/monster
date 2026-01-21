// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::prelude::{DateTime, Utc};
use matrix_sdk::authentication::oauth::registration::{
    ApplicationType, ClientMetadata, Localized, OAuthGrantType,
};
use matrix_sdk::authentication::oauth::{ClientRegistrationData, UrlOrQuery};
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::{
    authentication::matrix::MatrixSession,
    media::MediaFormat,
    ruma::{
        api::client::{error::StandardErrorBody, room::Visibility},
        events::{
            room::message::{MessageType, RoomMessageEventContent, TextMessageEventContent},
            AnyMessageLikeEventContent,
        },
        RoomId, UserId,
    },
    Client,
};
use matrix_sdk_ui::{
    eyeball_im::VectorDiff,
    sync_service::SyncService,
    timeline::{
        MsgLikeKind, TimelineBuilder, TimelineItemContent, TimelineItemKind, VirtualTimelineItem,
    },
};
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpSocket;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

use crate::room::Room;
use crate::roomlistitem::RoomListItem;
use crate::tombstone::RoomTombstoneEventContent;

mod room;
mod roomlistitem;
mod tombstone;

struct Connection {
    rt: Runtime,
    client: Client,
}

struct Rooms {
    queue: Arc<RwLock<Vec<VectorDiff<matrix_sdk_ui::room_list_service::RoomListItem>>>>,
}

struct RoomListVecDiff(VectorDiff<matrix_sdk_ui::room_list_service::RoomListItem>);

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

    fn item(&self) -> Box<RoomListItem> {
        match &self.0 {
            VectorDiff::Insert { value, .. } => Box::new(RoomListItem(value.clone())),
            VectorDiff::Set { value, .. } => Box::new(RoomListItem(value.clone())),
            VectorDiff::PushFront { value, .. } => Box::new(RoomListItem(value.clone())),
            VectorDiff::PushBack { value, .. } => Box::new(RoomListItem(value.clone())),
            _ => panic!(),
        }
    }

    fn items_vec(&self) -> Vec<RoomListItem> {
        match &self.0 {
            VectorDiff::Append { values, .. } => {
                values.iter().map(|ti| RoomListItem(ti.clone())).collect()
            }
            VectorDiff::Reset { values, .. } => {
                values.iter().map(|ti| RoomListItem(ti.clone())).collect()
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
                TimelineItemContent::MsgLike(message) => match &message.kind {
                    MsgLikeKind::Message(message) => message.body().to_string(),
                    MsgLikeKind::Sticker(sticker) => sticker.content().body.clone(),
                    MsgLikeKind::Poll(_) => "poll".to_string(),
                    MsgLikeKind::Redacted => "redacted".to_string(),
                    MsgLikeKind::UnableToDecrypt(_) => "utd".to_string(),
                    MsgLikeKind::Other(other) => format!("{:?}", other),
                },
                event => format!("{:?}", event),
            },
            TimelineItemKind::Virtual(virt) => match virt {
                VirtualTimelineItem::DateDivider(millis) => format!("{}", millis.0),
                VirtualTimelineItem::ReadMarker => "Readmarker".to_string(),
                VirtualTimelineItem::TimelineStart => "Timeline start".to_string(),
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
        let rt = Runtime::new().expect("Failed to create runtime");
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
        let rt = Runtime::new().expect("Failed to create runtime");
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

    pub(crate) fn init_oidc(server_name: String) -> Box<Connection> {
        let rt = Runtime::new().expect("Failed to create runtime");
        let client = rt.block_on(async {
            Client::builder()
                .server_name_or_homeserver_url(&server_name)
                .build()
                .await
                .unwrap()
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            let client = client_clone;
            let mut client_metadata = ClientMetadata::new(
                ApplicationType::Native,
                vec![OAuthGrantType::AuthorizationCode {
                    redirect_uris: vec![Url::parse("http://localhost").unwrap()],
                }],
                Localized::new(Url::parse("https://kde.org").unwrap(), None),
            );
            client_metadata.client_name = Some(Localized::new("Monster".to_string(), None));
            let oauth = client.oauth();
            let url = oauth
                .login(
                    Url::parse("http://localhost:18779").unwrap(),
                    None,
                    Some(ClientRegistrationData::new(
                        Raw::new(&client_metadata).unwrap(),
                    )),
                    None,
                )
                .build()
                .await
                .unwrap()
                .url
                .to_string();

            ffi::shim_oidc_login_url_available(server_name.clone(), url);

            let socket = TcpSocket::new_v4().unwrap();
            socket.bind("0.0.0.0:18779".parse().unwrap()).unwrap();

            let (mut stream, _) = socket.listen(1).unwrap().accept().await.unwrap();
            let mut data = String::new();

            stream
                .write_all("HTTP/1.0 200 OK\r\n\r\n".as_bytes())
                .await
                .unwrap();
            BufReader::new(stream).read_line(&mut data).await.unwrap();
            let query = &data.split(" ").nth(1).unwrap()[2..];
            oauth
                .finish_login(UrlOrQuery::Query(query.to_string()))
                .await
                .unwrap();
            ffi::shim_connected(server_name);

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
            let timeline = TimelineBuilder::new(&room).build().await.unwrap();
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
                let Some(entries) = stream.next().await else {
                    continue; //TODO or return?
                };

                for entry in entries {
                    queue.write().unwrap().push(entry);
                }
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
                Err(Api(error)) => match error.as_ref() {
                    Server(ClientApi(Error {
                        status_code: StatusCode::UNAUTHORIZED,
                        body:
                            ErrorBody::Standard(StandardErrorBody {
                                kind: ErrorKind::UnknownToken { .. },
                                ..
                            }),
                        ..
                    })) => {
                        ffi::shim_logged_out(client.user_id().unwrap().to_string());
                    }
                    _ => {}
                },
                Ok(..) => {
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
        Box::new(Room {
            room: self.client.get_room(&room_id).unwrap(),
        })
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

fn init_oidc(server_name: String) -> Box<Connection> {
    Connection::init_oidc(server_name)
}

fn restore(secret: String) -> Box<Connection> {
    Connection::restore(secret)
}

// NOTE: When adding functions here, delete the entire build folder. There's probably something missing somewhere to make the header regenerate automatically
#[cxx::bridge]
mod ffi {
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

        fn shim_connected(matrix_id: String);
        fn shim_rooms_changed(matrix_id: String);
        fn shim_timeline_changed(matrix_id: String, room_id: String);
        fn shim_avatar_loaded(room_id: String, data: Vec<u8>);
        fn shim_logged_out(matrix_id: String);

        fn shim_oidc_login_url_available(server_name: String, url: String);
    }
}
