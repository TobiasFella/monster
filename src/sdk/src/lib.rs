// SPDX-FileCopyrightText: 2025 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::prelude::{DateTime, Utc};
use matrix_sdk::authentication::oauth::{ClientId, UserSession};
use matrix_sdk::{
    authentication::matrix::MatrixSession,
    ruma::{
        api::client::{room::Visibility},
        events::{
            room::message::{MessageType, RoomMessageEventContent, TextMessageEventContent},
            AnyMessageLikeEventContent,
        },
        UserId,
    },
};
use matrix_sdk_ui::{
    eyeball_im::VectorDiff,
    timeline::{
        MsgLikeKind, TimelineItemContent, TimelineItemKind, VirtualTimelineItem,
    },
};
use std::sync::{Arc, RwLock};
use matrix_sdk::ruma::exports::serde::{Deserialize, Serialize};

use crate::connection::Connection;
use crate::room::Room;
use crate::roomlistitem::RoomListItem;
use crate::tombstone::RoomTombstoneEventContent;

mod room;
mod roomlistitem;
mod tombstone;
mod connection;

mod ffi;

#[derive(Serialize, Deserialize)]
struct SessionData {
    oidc: Option<OidcSession>,
    native: Option<MatrixSession>,
}

#[derive(Serialize, Deserialize)]
struct OidcSession {
    client_id: ClientId,
    user_session: UserSession,
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
