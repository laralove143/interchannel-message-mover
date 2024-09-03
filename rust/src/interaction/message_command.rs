use anyhow::Result;
use sparkle_convenience::{error::IntoError, interaction::extract::InteractionDataExt};
use twilight_model::{channel::Message, guild::Permissions};

use crate::{interaction::InteractionContext, CustomError, REQUIRED_PERMISSIONS};

impl InteractionContext<'_> {
    pub fn handle_message_command(&self) -> Result<Message> {
        self.handle.check_permissions(REQUIRED_PERMISSIONS)?;

        let message = self
            .interaction
            .data
            .clone()
            .ok()?
            .command()
            .ok()?
            .resolved
            .ok()?
            .messages
            .into_iter()
            .next()
            .ok()?
            .1;
        let member = self.interaction.member.as_ref().ok()?;
        let user = member.user.as_ref().ok()?;

        if message.author.id != user.id
            && !member
                .permissions
                .ok()?
                .contains(Permissions::MANAGE_MESSAGES)
        {
            return Err(CustomError::ManageMessagesPermissionsMissing.into());
        }

        Ok(message)
    }
}
