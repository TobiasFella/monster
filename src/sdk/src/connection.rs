// SPDX-FileCopyrightText: 2026 Tobias Fella <tobias.fella@kde.org>
// SPDX-License-Identifier: LGPL-2.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;
use matrix_sdk::authentication::oauth::{ClientRegistrationData, OAuthSession, UrlOrQuery};
use matrix_sdk::Client;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::ruma::api::client::error::StandardErrorBody;
use matrix_sdk::ruma::{RoomId, UserId};
use tokio::runtime::Runtime;
use crate::{ffi::ffi, OidcSession, RoomCreateOptions, Rooms, SessionData, Timeline};
use crate::room::Room;
use std::sync::{Arc, RwLock};
use eyeball_im::VectorDiff;
use matrix_sdk::authentication::oauth::registration::{ApplicationType, ClientMetadata, Localized, OAuthGrantType};
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk_ui::sync_service::SyncService;
use matrix_sdk_ui::timeline::TimelineBuilder;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpSocket;
use tokio_stream::StreamExt;

pub(crate) struct Connection {
    pub rt: Runtime,
    pub client: Client,
}

fn sqlite_passphrase<'a>() -> Option<&'a str> {
    None //TODO
}

fn token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

fn state_parent() -> PathBuf {
    dirs::state_dir()
        .unwrap()
        .join("Arctic")
        .join("monster")
}

fn state_dir(matrix_id: &String) -> PathBuf {
    state_parent()
        .join(matrix_id)
}

fn needs_resolving(matrix_id: String) -> Option<String> {
    let file = state_parent().join("unresolved.json");
    if !std::fs::exists(&file).unwrap() {
        None
    } else {
        serde_json::from_str::<HashMap<String, String>>(&std::fs::read_to_string(file).unwrap()).unwrap().get(&matrix_id).map(|it| it.to_string())
    }
}

fn remove_from_unresolved(matrix_id: String) {
    let file = state_parent().join("unresolved.json");
    let mut unresolved = serde_json::from_str::<HashMap<String, String>>(&std::fs::read_to_string(&file).unwrap()).unwrap();
    unresolved.remove(&matrix_id);
    std::fs::write(file, serde_json::to_string(&unresolved).unwrap()).unwrap();
}

fn add_to_unresolved(matrix_id: String, token: String) {
    let file = state_parent().join("unresolved.json");
    std::fs::create_dir_all(state_parent()).unwrap();
    let mut unresolved: HashMap<String, String> = if let Ok(data) = std::fs::read_to_string(&file) {
        serde_json::from_str::<HashMap<String, String>>(&data).unwrap_or_default()
    } else {
        Default::default()
    };
    unresolved.insert(matrix_id, token);
    std::fs::write(file, serde_json::to_string(&unresolved).unwrap()).unwrap();
}

impl Connection {
    pub(crate) fn restore(secret: String) -> Box<Connection> {
        let session_data: SessionData = serde_json::from_str(&secret).unwrap();
        let rt = Runtime::new().expect("Failed to create runtime");

        let matrix_id = if session_data.oidc.is_some() {
            let data = session_data.oidc.as_ref().unwrap();
            let matrix_id = data.user_session.meta.user_id.clone();

            if let Some(token) = needs_resolving(matrix_id.to_string()) {
                std::fs::rename(state_dir(&token), state_dir(&matrix_id.to_string())).unwrap();
                remove_from_unresolved(matrix_id.to_string());
            }
            matrix_id
        } else {
            session_data.native.as_ref().unwrap().meta.user_id.clone()
        };

        let client = rt.block_on(async {
            Client::builder()
                .server_name(matrix_id.server_name())
                .sqlite_store(
                    state_dir(&matrix_id.to_string()),
                    sqlite_passphrase(),
                )
                .handle_refresh_tokens()
                .build()
                .await
                .unwrap()
        });
        let client_clone = client.clone();
        rt.spawn(async move {
            if session_data.oidc.is_some() {
                let oidc = session_data.oidc.unwrap();
                client_clone.restore_session(OAuthSession {
                    client_id: oidc.client_id,
                    user: oidc.user_session,
                }).await.unwrap();
            } else {
                client_clone.restore_session(session_data.native.unwrap()).await.unwrap();
            }
            ffi::shim_connected(matrix_id.to_string());
        });
        Box::new(Connection { rt, client })
    }

    pub(crate) fn timeline_paginate_back(&self, timeline: &Timeline) {
        let timeline = timeline.timeline.clone();
        self.rt.spawn(async move {
            timeline.write().await.paginate_backwards(20).await.unwrap();
        });
    }

    pub(crate) fn session(&self) -> String {
        use matrix_sdk::AuthSession;
        let data = match self.client.session().unwrap() {
            AuthSession::Matrix(session) => SessionData {
                oidc: None,
                native: Some(session),
            },
            AuthSession::OAuth(session) => SessionData {
                oidc: Some(OidcSession {
                    client_id: session.client_id,
                    user_session: session.user,
                }),
                native: None,
            },
            _ => panic!("Unexpected auth session type"),
        };
        serde_json::to_string(&data).unwrap()
    }

    pub(crate) fn init(matrix_id: String, password: String) -> Box<Connection> {
        let rt = Runtime::new().expect("Failed to create runtime");
        let _ =
            std::fs::remove_dir_all(state_dir(&matrix_id));
        let client = rt.block_on(async {
            let user_id = UserId::parse(&matrix_id).unwrap();
            Client::builder()
                .server_name(&user_id.server_name())
                .sqlite_store(
                    state_dir(&matrix_id),
                    sqlite_passphrase(),
                )
                .handle_refresh_tokens()
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
        let token = token();

        let rt = Runtime::new().expect("Failed to create runtime");
        let client = rt.block_on(async {
            Client::builder()
                .server_name_or_homeserver_url(&server_name)
                .sqlite_store(state_dir(&token), sqlite_passphrase())
                .handle_refresh_tokens()
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
            add_to_unresolved(client.user_id().unwrap().to_string(), token);
            ffi::shim_connected(server_name);
        });
        Box::new(Connection { rt, client })
    }

    pub(crate) fn timeline(&self, room_id: String) -> Box<Timeline> {
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

    pub(crate) fn room_avatar(&self, room_id: String) {
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

    pub(crate) fn device_id(&self) -> String {
        self.client.device_id().unwrap().to_string()
    }

    pub(crate) fn matrix_id(&self) -> String {
        self.client.user_id().unwrap().to_string()
    }

    pub(crate) fn slide(&self) -> Box<Rooms> {
        let client = self.client.clone();

        let clone = client.clone();
        self.rt.spawn(async move {
            let client = clone;
            let mut devices = client.encryption().devices_stream().await.unwrap();
            use tokio::pin;
            pin!(devices);

            for entry in devices.next().await {
                println!("{:?}", entry);
            }
        });

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

    pub(crate) fn logout(&self) {
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

    pub(crate) fn create_room(&self, room_create_options: &RoomCreateOptions) {
        let client = self.client.clone();
        let options = room_create_options.0.clone();
        self.rt.spawn(async move {
            client.create_room(options).await.unwrap();
        });
    }

    pub(crate) fn room(&self, id: String) -> Box<Room> {
        println!("Looking up room {}", id);
        // TODO: This seems to sometimes return None even when we are joined to the room; figure out why
        // Leads to a crash on startup, probably the initial sync isn't completed yet?
        let room_id = RoomId::parse(id).unwrap();
        Box::new(Room {
            room: self.client.get_room(&room_id).unwrap(),
        })
    }

    pub(crate) fn is_known_room(&self, id: String) -> bool {
        let room_id = RoomId::parse(id).unwrap();
        self.client.get_room(&room_id).is_some()
    }

    pub(crate) fn set_display_name(&self, display_name: String) -> String {
        let token = token();
        let client = self.client.clone();
        let token_clone = token.clone();
        self.rt.spawn(async move {
            client.refresh_access_token().await.unwrap(); //TODO????
            client.account().set_display_name(Some(&display_name)).await.unwrap();
            ffi::task_done(token_clone);
        });
        token
    }
}