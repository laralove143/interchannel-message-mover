use anyhow::Result;
use sparkle_convenience::error::IntoError;
use twilight_model::channel::{Channel, Message};

use crate::{Context, CustomError};

impl Context {
    pub async fn execute_webhook_as_member(
        &self,
        message: &Message,
        channel: &Channel,
    ) -> Result<()> {
        let mut channel_id = channel.id;
        let mut thread_id = None;
        if channel.kind.is_thread() {
            thread_id = Some(channel_id);
            channel_id = channel.parent_id.ok()?;
        };

        let webhook = match self
            .bot
            .http
            .channel_webhooks(channel_id)
            .await?
            .models()
            .await?
            .into_iter()
            .find(|webhook| webhook.token.is_some())
        {
            Some(webhook) => webhook,
            None => {
                self.bot
                    .http
                    .create_webhook(channel_id, "interchannel message mover")?
                    .await?
                    .model()
                    .await?
            }
        };
        let webhook_token = webhook.token.ok()?;

        let mut execute_webhook = self
            .bot
            .http
            .execute_webhook(webhook.id, &webhook_token)
            .content(&message.content)
            .map_err(|_| CustomError::MessageTooLong)?
            .username(
                message
                    .member
                    .as_ref()
                    .and_then(|member| member.nick.as_ref())
                    .unwrap_or(&message.author.name),
            )?;

        if let Some(thread_id) = thread_id {
            execute_webhook = execute_webhook.thread_id(thread_id);
        }

        if let Some(avatar_url) = message
            .member
            .as_ref()
            .and_then(|member| member.avatar)
            .zip(message.guild_id)
            .map(|(avatar, guild_id)| {
                format!(
                    "https://cdn.discordapp.com/guilds/{guild_id}/users/{}/avatar/{}.png",
                    message.author.id, avatar
                )
            })
            .or_else(|| {
                message.author.avatar.map(|avatar| {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        message.author.id, avatar
                    )
                })
            })
        {
            execute_webhook.avatar_url(&avatar_url).await?;
        } else {
            execute_webhook.await?;
        }

        Ok(())
    }
}

pub fn check(message: &Message) -> Result<()> {
    if !message.attachments.is_empty() {
        return Err(CustomError::MessageAttachment.into());
    }

    Ok(())
}
