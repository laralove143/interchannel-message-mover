use anyhow::Result;
use sparkle_convenience::reply::Reply;
use twilight_model::application::command::{Command, CommandType};
use twilight_util::builder::command::CommandBuilder;

use crate::{interaction::InteractionContext, message};

pub const NAME: &str = "move message";

pub fn command() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message)
        .dm_permission(false)
        .build()
}

impl InteractionContext<'_> {
    pub async fn handle_move_message_command(self) -> Result<()> {
        let message = self.handle_message_command()?;
        message::check(&message)?;

        let message_id = message.id;
        let message_channel_id = message.channel_id;

        let channel = self.wait_for_channel_select_interaction().await?;

        self.handle
            .reply(
                Reply::new()
                    .ephemeral()
                    .update_last()
                    .content("starting up the bike :motor_scooter:"),
            )
            .await?;

        self.ctx
            .execute_webhook_as_member(&message, &channel)
            .await?;
        self.ctx
            .bot
            .http
            .delete_message(message_channel_id, message_id)
            .await?;

        self.handle
            .reply(
                Reply::new()
                    .ephemeral()
                    .update_last()
                    .content("done :incoming_envelope:"),
            )
            .await?;

        Ok(())
    }
}
