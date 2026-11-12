use serde::Serialize;

use crate::{
    model::chat::{ChannelDetails, ChatChannel, ChatMessage, NewConversation, UserSilence},
    request::{JsonBody, Query, Request},
    routing::Route,
    Osu,
};

/// Reset the chat activity timeout.
///
/// Send this request periodically to reset the chat activity timeout. This
/// also returns a [`Vec`] of recent [`UserSilence`]s.
///
/// Note that the client has to be initialized with the
/// [`ChatRead`](crate::prelude::Scopes::ChatRead) scope through the OAuth
/// process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct KeepaliveChat<'a> {
    osu: &'a Osu,
}

impl<'a> KeepaliveChat<'a> {
    pub(crate) const fn new(osu: &'a Osu) -> Self {
        Self { osu }
    }
}

into_future! {
    |self: KeepaliveChat<'_>| -> Vec<UserSilence> {
        Request::new(Route::PostChatAck)
    }
}

/// Create a new PM conversation.
///
/// This is the equivalent of `/msg` on IRC and in-game. This returns a
/// [`NewConversation`], which includes the created [`ChatChannel`] and
/// the sent [`ChatMessage`].
///
/// To create a PM without sending a message, use the [`CreateChannel`] request.
///
/// Note that the client has to be initialized with the
/// [`ChatWrite`](crate::prelude::Scopes::ChatWrite) scope through the OAuth
/// process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
#[derive(Serialize)]
pub struct CreateNewConversation<'a> {
    #[serde(skip)]
    osu: &'a Osu,
    target_id: u32,
    message: Box<str>,
    is_action: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    uuid: Option<Box<str>>,
}

impl<'a> CreateNewConversation<'a> {
    #[inline]
    pub(crate) fn new(osu: &'a Osu, target_id: u32, message: impl Into<String>) -> Self {
        Self {
            osu,
            target_id,
            message: Box::from(message.into()),
            is_action: false,
            uuid: None,
        }
    }

    #[inline]
    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.uuid = Some(Box::from(uuid.into()));

        self
    }

    #[inline]
    pub const fn as_action(mut self) -> Self {
        self.is_action = true;

        self
    }
}

into_future! {
    |self: CreateNewConversation<'_>| -> NewConversation {
        let mut body = JsonBody::new();

        body.push_int("target_id", self.target_id);
        body.push_str("message", self.message.as_ref());
        body.push_bool("is_action", self.is_action);

        if let Some(uuid) = &self.uuid {
            body.push_str("uuid", uuid.as_ref());
        }

        Request::with_body(Route::PostChatNew, body)
    }
}

/// Get a [`Vec`] of [`ChatMessage`]s for a specific channel.
///
/// This is typically used to batch prefetch messages for a channel before
/// actively listening for them via WebSocket afterwards.
///
/// Note that the client has to be initialized with the
/// [`ChatRead`](crate::prelude::Scopes::ChatRead) scope through the OAuth
/// process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
#[derive(Serialize)]
pub struct GetChannelMessages<'a> {
    #[serde(skip)]
    osu: &'a Osu,
    #[serde(skip)]
    channel_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<u64>,
}

impl<'a> GetChannelMessages<'a> {
    #[inline]
    pub(crate) const fn new(osu: &'a Osu, channel_id: u64) -> Self {
        Self {
            osu,
            channel_id,
            limit: None,
            since: None,
            until: None,
        }
    }

    #[inline]
    pub const fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);

        self
    }

    #[inline]
    pub const fn since(mut self, since: u64) -> Self {
        self.since = Some(since);

        self
    }

    #[inline]
    pub const fn until(mut self, until: u64) -> Self {
        self.until = Some(until);

        self
    }
}

into_future! {
    |self: GetChannelMessages<'_>| -> Vec<ChatMessage> {
        Request::with_query(Route::GetChannelMessages { channel_id: self.channel_id }, Query::encode(&self))
    }
}

/// Send a [`ChatMessage`] to a specific channel and return it.
///
/// Note that the client has to be initialized with the
/// [`ChatWrite`](crate::prelude::Scopes::ChatWrite) scope through the OAuth
/// process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
#[derive(Serialize)]
pub struct SendMessage<'a> {
    #[serde(skip)]
    osu: &'a Osu,
    #[serde(skip)]
    channel_id: u64,
    message: Box<str>,
    is_action: bool,
}

/// Send a message to a channel.
///
/// This returns the sent [`ChatMessage`].
///
/// When sending a message, it is automatically marked as read.
///
/// Note that the client has to be initialized with the
/// [`ChatWrite`](crate::prelude::Scopes::ChatWrite) scope through the OAuth
/// process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
impl<'a> SendMessage<'a> {
    pub(crate) fn new(osu: &'a Osu, channel_id: u64, message: impl Into<String>) -> Self {
        Self {
            osu,
            channel_id,
            message: Box::from(message.into()),
            is_action: false,
        }
    }

    pub const fn as_action(mut self) -> Self {
        self.is_action = true;

        self
    }
}

into_future! {
    |self: SendMessage<'_>| -> ChatMessage {
        let mut body = JsonBody::new();

        body.push_str("message", self.message.as_ref());
        body.push_bool("is_action", self.is_action);

        Request::with_body(Route::PostChannelMessages { channel_id: self.channel_id }, body)
    }
}

/// Join a public or multiplayer [`ChatChannel`] and returns it.
///
/// This is the equivalent of `/join` on IRC and in-game.
///
/// Note that the client has to be initialized with the
/// [`ChatWriteManage`](crate::prelude::Scopes::ChatWriteManage) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct JoinChannel<'a> {
    osu: &'a Osu,
    channel_id: u64,
    user_id: u32,
}

impl<'a> JoinChannel<'a> {
    pub(crate) const fn new(osu: &'a Osu, channel_id: u64, user_id: u32) -> Self {
        Self {
            osu,
            channel_id,
            user_id,
        }
    }
}

into_future! {
    |self: JoinChannel<'_>| -> ChatMessage {
        Request::new(Route::PutChannelUser { channel_id: self.channel_id, user_id: self.user_id })
    }
}

/// Leave a public channel.
///
/// Note that the client has to be initialized with the
/// [`ChatWriteManage`](crate::prelude::Scopes::ChatWriteManage) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct LeaveChannel<'a> {
    osu: &'a Osu,
    channel_id: u64,
    user_id: u32,
}

impl<'a> LeaveChannel<'a> {
    pub(crate) const fn new(osu: &'a Osu, channel_id: u64, user_id: u32) -> Self {
        Self {
            osu,
            channel_id,
            user_id,
        }
    }
}

into_future! {
    |self: LeaveChannel<'_>| -> () {
        Request::new(Route::DeleteChannelUser { channel_id: self.channel_id, user_id: self.user_id })
    }
}

/// Mark a channel as being read up to a given `message_id`.
///
/// Note that the client has to be initialized with the
/// [`ChatRead`](crate::prelude::Scopes::ChatRead) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct MarkChannelAsRead<'a> {
    osu: &'a Osu,
    channel_id: u64,
    message_id: u64,
}

impl<'a> MarkChannelAsRead<'a> {
    pub(crate) const fn new(osu: &'a Osu, channel_id: u64, message_id: u64) -> Self {
        Self {
            osu,
            channel_id,
            message_id,
        }
    }
}

into_future! {
    |self: MarkChannelAsRead<'_>| -> () {
        Request::new(Route::PutChannelMarkAsRead { channel_id: self.channel_id, message_id: self.message_id })
    }
}

/// Get a [`Vec`] of all joinable public [`ChatChannel`]s.
///
/// Note that the client has to be initialized with the
/// [`ChatRead`](crate::prelude::Scopes::ChatRead) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct GetChannelList<'a> {
    osu: &'a Osu,
}

impl<'a> GetChannelList<'a> {
    pub(crate) const fn new(osu: &'a Osu) -> Self {
        Self { osu }
    }
}

into_future! {
    |self: GetChannelList<'_>| -> Vec<ChatChannel> {
        Request::new(Route::GetChannels)
    }
}

/// Create a [`ChatChannel`].
///
/// This request is currently limited to creating `PM` channels only (with the
/// option of not sending a mandatory message), as `ANNOUNCE` channels can only
/// be created by moderators and chat announcers.
///
/// Note that the client has to be initialized with the
/// [`ChatWriteManage`](crate::prelude::Scopes::ChatWriteManage) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct CreateChannel<'a> {
    osu: &'a Osu,
    target_id: u32,
    message: Option<Box<str>>,
}

impl<'a> CreateChannel<'a> {
    pub(crate) const fn new(osu: &'a Osu, target_id: u32) -> Self {
        Self {
            osu,
            target_id,
            message: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(Box::from(message.into()));

        self
    }
}

into_future! {
    |self: CreateChannel<'_>| -> ChatChannel {
        let mut body = JsonBody::new();

        // type "ANNOUNCE" is reserved to moderators and "chat announcers"
        // (whatever it means). realistically they should have some better
        // internal tooling
        body.push_str("type", "PM");
        body.push_int("target_id", self.target_id);

        if let Some(message) = &self.message {
            body.push_str("message", message.as_ref());
        }

        Request::with_body(Route::PostChannel, body)
    }
}

/// Get the a [`ChannelDetails`].
///
/// Note that the client has to be initialized with the
/// [`ChatRead`](crate::prelude::Scopes::ChatRead) scope through
/// the OAuth process in order for this endpoint to not return an error.
///
/// See [`OsuBuilder::with_authorization`](crate::OsuBuilder::with_authorization).
#[must_use = "requests must be configured and executed"]
pub struct GetChannel<'a> {
    osu: &'a Osu,
    channel_id: u64,
}

impl<'a> GetChannel<'a> {
    pub(crate) const fn new(osu: &'a Osu, channel_id: u64) -> Self {
        Self { osu, channel_id }
    }
}

into_future! {
    |self: GetChannel<'_>| -> ChannelDetails {
        Request::new(Route::GetChannel { channel_id: self.channel_id })
    }
}
