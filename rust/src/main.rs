#![warn(clippy::nursery, clippy::pedantic)]

use std::{env, sync::Arc};

use anyhow::Result;
use futures::StreamExt;
use sparkle_convenience::{
    error::{ErrorExt, UserError},
    log::DisplayFormat,
    prettify::Prettify,
    reply::Reply,
    Bot,
};
use twilight_gateway::{
    error::ReceiveMessageErrorType, stream::ShardEventStream, EventTypeFlags, Intents,
};
use twilight_model::{
    gateway::event::Event,
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};
use twilight_standby::Standby;

use crate::interaction::set_commands;

mod interaction;
mod message;

const TEST_GUILD_ID: Id<GuildMarker> = Id::new(903_367_565_349_384_202);
const LOGGING_CHANNEL_ID: Id<ChannelMarker> = Id::new(1_002_953_459_890_397_287);

const REQUIRED_PERMISSIONS: Permissions = Permissions::MANAGE_WEBHOOKS
    .union(Permissions::VIEW_CHANNEL)
    .union(Permissions::MANAGE_MESSAGES)
    .union(Permissions::READ_MESSAGE_HISTORY);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CustomError {
    #[error("you need **Manage Messages** permissions to move messages that are not your own")]
    ManageMessagesPermissionsMissing,
    #[error(
        "you need **Send Messages** permissions in the channel you want to move the messages to"
    )]
    SendMessagesPermissionMissing,
    #[error("one of the messages is too long, you're probably using your super nitro powers")]
    MessageTooLong,
    #[error(
        "one of the messages has an image/file, this will be handled better in the future, sorry \
         for the inconvenience and stay tuned!"
    )]
    MessageAttachment,
    #[error(
        "i can work with up to 50 messages, if you need a higher limit, please join the support \
         server and tell lara"
    )]
    TooManyMessages,
    #[error(
        "i can't work with messages older than 2 weeks, if you need me to, please join the \
         support server and tell lara"
    )]
    MessageTooOld,
}

struct Context {
    bot: Bot,
    standby: Standby,
}

impl Context {
    async fn handle_event(&self, event: Event) {
        self.standby.process(&event);

        if let Event::InteractionCreate(interaction) = event {
            self.handle_interaction(interaction.0).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let (mut bot, mut shards) = Bot::new(
        env::var("BOT_TOKEN")?,
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE,
    )
    .await?;
    bot.set_logging_format(DisplayFormat::Debug);
    bot.set_logging_channel(LOGGING_CHANNEL_ID).await?;
    bot.set_logging_file("logs.txt".to_owned());

    set_commands(&bot).await?;

    let ctx = Arc::new(Context {
        bot,
        standby: Standby::new(),
    });

    let mut events = ShardEventStream::new(shards.iter_mut());
    while let Some((_, event_res)) = events.next().await {
        let ctx_ref = Arc::clone(&ctx);
        match event_res {
            Ok(event) => {
                tokio::spawn(async move {
                    ctx_ref.handle_event(event).await;
                });
            }
            Err(err)
                if !matches!(
                    err.kind(),
                    ReceiveMessageErrorType::Deserializing { .. } | ReceiveMessageErrorType::Io
                ) =>
            {
                ctx_ref.bot.log(&err).await;

                if err.is_fatal() {
                    break;
                }
            }
            Err(_) => (),
        }
    }

    Ok(())
}

fn err_reply(err: &anyhow::Error) -> Reply {
    let message = if let Some(UserError::MissingPermissions(permissions)) = err.user() {
        format!(
            "please beg the mods to give me these permissions first:\n{}",
            permissions.unwrap_or(REQUIRED_PERMISSIONS).prettify()
        )
    } else if let Some(custom_err) = err.downcast_ref::<CustomError>() {
        custom_err.to_string()
    } else {
        "something went terribly wrong there... i spammed lara (the dev) with the error, im sure \
         they'll look at it asap"
            .to_owned()
    };

    Reply::new().ephemeral().update_last().content(message)
}
