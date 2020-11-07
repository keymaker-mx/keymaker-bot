use matrix_sdk::{
    events::{
        room::message::{InReplyTo, MessageEventContent, RelatesTo},
        AnyMessageEventContent, SyncMessageEvent,
    },
    identifiers::EventId,
    Room,
};
use tracing::*;

pub trait AnyMessageEventContentExt {
    fn add_relates_to(&mut self, new_relates_to: EventId);
}

impl AnyMessageEventContentExt for AnyMessageEventContent {
    fn add_relates_to(&mut self, new_relates_to: EventId) {
        if let AnyMessageEventContent::RoomMessage(message) = self {
            if let MessageEventContent::Notice(notice) = message {
                notice.relates_to = Some(RelatesTo {
                    in_reply_to: Some(InReplyTo {
                        event_id: new_relates_to,
                    }),
                });
            }
        }
    }
}

pub trait RoomExt {
    fn get_sender_displayname<'a>(
        &'a self,
        event: &'a SyncMessageEvent<MessageEventContent>,
    ) -> &'a str;
}

impl RoomExt for Room {
    #[instrument]
    fn get_sender_displayname<'a>(
        &'a self,
        event: &'a SyncMessageEvent<MessageEventContent>,
    ) -> &'a str {
        self.joined_members
            .get(&event.sender)
            .or_else(|| self.invited_members.get(&event.sender))
            .and_then(|member| member.display_name.as_deref())
            .unwrap_or_else(|| event.sender.as_str())
    }
}
