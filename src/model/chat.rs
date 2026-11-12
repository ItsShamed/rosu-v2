use std::{fmt, str::FromStr};

use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize,
};
use time::OffsetDateTime;

use crate::{
    error::{OsuError, ParsingError},
    model::{user::User, CacheUserFn, ContainedUsers},
    request::{GetChannelMessages, JoinChannel, LeaveChannel, MarkChannelAsRead, SendMessage},
    Osu,
};

use super::serde_util;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct UserSilence {
    /// identifier for this silence
    pub id: u32,
    /// the user that was silenced
    pub user_id: u32,
}

impl ContainedUsers for UserSilence {
    fn apply_to_users(&self, _: impl CacheUserFn) {}
}

/// Represents the type of a message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageType {
    /// Represents a message in plain text.
    Plain,
    /// Represents a message that is formatted using Markdown.
    Markdown,
    /// Represents a message that user is supposedely acting, such as when using
    /// the `/me` or `/np` commands.
    Action,
}

impl fmt::Display for MessageType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for MessageType {
    type Err = OsuError;

    fn from_str(ty: &str) -> Result<Self, Self::Err> {
        let message_type = match ty.to_lowercase().as_str() {
            "plain" => Self::Plain,
            "markdown" => Self::Markdown,
            "action" => Self::Action,
            _ => return Err(ParsingError::MessageType(ty.into()).into()),
        };

        Ok(message_type)
    }
}

struct MessageTypeVisitor;

impl Visitor<'_> for MessageTypeVisitor {
    type Value = MessageType;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        MessageType::from_str(v)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &"a message type string"))
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(MessageTypeVisitor)
    }
}

/// Represents the type of a channel
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelType {
    /// Represents a publicly accessible channel to everyone (e.g. `#osu`, or
    /// `#lazer`)
    Public,
    /// Represents a channel that is accessible only to certain groups
    #[doc(hidden)]
    // most likely used by admins and moderators (GMT, NAT, etc.)
    Private,
    /// Represents a chat channel from a multiplayer lobby.
    Multiplayer,
    /// Represents a chat channel for spectators of a player.
    #[doc(hidden)]
    // neither osu!(lazer) nor osu!web currently have proper support for
    // spectator channels yet
    Spectator,
    /// Represents a Private Message channel.
    PM,
    /// Represents a group chat channel, such as team chat.
    Group,
    /// Represents an announcement channel (e.g. `#announce`)
    Announce,
}

impl fmt::Display for ChannelType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for ChannelType {
    type Err = OsuError;

    fn from_str(ty: &str) -> Result<Self, Self::Err> {
        let message_type = match ty.to_uppercase().as_str() {
            "PUBLIC" => Self::Public,
            "PRIVATE" => Self::Private,
            "MULTIPLAYER" => Self::Multiplayer,
            "SPECTATOR" => Self::Spectator,
            "PM" => Self::PM,
            "GROUP" => Self::Group,
            "ANNOUNCE" => Self::Announce,
            _ => return Err(ParsingError::ChannelType(ty.into()).into()),
        };

        Ok(message_type)
    }
}

struct ChannelTypeVisitor;

impl Visitor<'_> for ChannelTypeVisitor {
    type Value = ChannelType;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        ChannelType::from_str(v)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &"a channel type string"))
    }
}

impl<'de> Deserialize<'de> for ChannelType {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(ChannelTypeVisitor)
    }
}

/// Represents a message sent in the in-game chat.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ChatMessage {
    /// Unique identifier for this message
    #[serde(rename = "message_id")]
    pub id: u64,
    /// The identifier of the channel it was sent in
    pub channel_id: u64,
    /// The identifier of the user that sent this message
    pub sender_id: u32,
    /// Message identifier originally sent by the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Whether this message is an action or not, i.e. when using `/me` or `/np`
    pub is_action: bool,
    /// The type of this message
    #[serde(rename = "type")]
    pub message_type: MessageType,
    /// The time and date this message was sent
    #[serde(with = "serde_util::datetime")]
    pub timestamp: OffsetDateTime,
    /// The content of this message
    pub content: String,
    /// The user that sent this message. Sometimes embedded to save additionnal
    /// lookups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

impl ContainedUsers for ChatMessage {
    fn apply_to_users(&self, f: impl super::CacheUserFn) {
        if let Some(user) = &self.user {
            user.apply_to_users(f);
        }
    }
}

/// Represents attributes tied to this user w.r.t a particular [`ChatChannel`]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ChatChannelUserAttributes {
    pub can_message: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_message_error: Option<String>,
    pub last_read_id: u64,
}

/// Represents a chat channel.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ChatChannel {
    /// Unique identifier for this channel
    #[serde(rename = "channel_id")]
    pub id: u64,
    /// The name of this channel
    pub name: String,
    /// The optional description of this channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The icon path for this channel (root is <https://osu.ppy.sh>)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// The type of this channel
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// The maximum length of a message in this channel
    pub message_length_limit: u32,
    /// Whether the user is forbidden to send messages in this channel
    pub moderated: bool,
    /// Unique identifier originally sent by the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// The number of users that joined this channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_user_count: Option<u32>,
    /// Attributes for the current user w.r.t to this channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_user_attributes: Option<ChatChannelUserAttributes>,
    /// The identifier of the last message that was sent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<u64>,
    /// The list of user identifiers that joined this channel (not included
    /// for [`Public`](ChannelType::Public) channels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Vec<u32>,
}

impl ChatChannel {
    /// Join this channel.
    #[inline]
    pub const fn join<'o>(&self, osu: &'o Osu, user_id: u32) -> JoinChannel<'o> {
        osu.channel_join(self.id, user_id)
    }

    /// Leave this channel.
    #[inline]
    pub const fn leave<'o>(&self, osu: &'o Osu, user_id: u32) -> LeaveChannel<'o> {
        osu.channel_leave(self.id, user_id)
    }

    /// Mark this channel as read up to the given `message_id`.
    #[inline]
    pub const fn mark_as_read<'o>(&self, osu: &'o Osu, message_id: u64) -> MarkChannelAsRead<'o> {
        osu.channel_mark_as_read(self.id, message_id)
    }

    /// Get a list of messages sent in this channel.
    #[inline]
    pub const fn messages<'o>(&self, osu: &'o Osu) -> GetChannelMessages<'o> {
        osu.channel_messages(self.id)
    }

    /// Send a message to this channel
    #[inline]
    pub fn send_message<'o>(&self, osu: &'o Osu, message: impl Into<String>) -> SendMessage<'o> {
        osu.send_message(self.id, message)
    }
}

impl ContainedUsers for ChatChannel {
    fn apply_to_users(&self, _: impl super::CacheUserFn) {}
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct NewConversation {
    pub channel: ChatChannel,
    pub message: ChatMessage,
}

impl ContainedUsers for NewConversation {
    fn apply_to_users(&self, f: impl super::CacheUserFn) {
        self.message.apply_to_users(f);
    }
}

/// Represents the details for a specific [`ChatChannel`]
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ChannelDetails {
    /// The [`ChatChannel`] in question.
    pub channel: ChatChannel,
    /// The list of [`User`]s in this channels, populated for `PM` channels
    /// only.
    pub users: Vec<User>,
}

impl ContainedUsers for ChannelDetails {
    fn apply_to_users(&self, f: impl CacheUserFn) {
        for user in &self.users {
            user.apply_to_users(f);
        }
    }
}
