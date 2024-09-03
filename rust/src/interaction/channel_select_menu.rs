use anyhow::Result;
use serde::Serialize;
use sparkle_convenience::interaction::DeferVisibility;
use twilight_http::Response;
use twilight_model::channel::{
    message::{component::ComponentType, MessageFlags},
    ChannelType, Message,
};

use crate::interaction::InteractionContext;

const CHANNEL_SELECT_MENU_TYPE: u8 = 8;

#[derive(Serialize)]
pub struct ChannelSelectMenu {
    #[serde(rename = "type")]
    kind: u8,
    custom_id: String,
    channel_types: Vec<ChannelType>,
}

impl ChannelSelectMenu {
    pub fn new(custom_id: String, channel_types: Vec<ChannelType>) -> Self {
        Self {
            kind: CHANNEL_SELECT_MENU_TYPE,
            custom_id,
            channel_types,
        }
    }
}

#[derive(Serialize)]
struct ActionRow {
    #[serde(rename = "type")]
    kind: u8,
    components: Vec<ChannelSelectMenu>,
}

#[derive(Serialize)]
struct InteractionResponse {
    content: String,
    flags: Option<MessageFlags>,
    components: Vec<ActionRow>,
}

impl InteractionContext<'_> {
    pub async fn followup_with_channel_select_menu(
        &self,
        content: String,
        visibility: DeferVisibility,
        menu: ChannelSelectMenu,
    ) -> Result<Response<Message>> {
        let response = InteractionResponse {
            content,
            flags: (visibility == DeferVisibility::Ephemeral).then_some(MessageFlags::EPHEMERAL),
            components: vec![ActionRow {
                kind: ComponentType::ActionRow.into(),
                components: vec![menu],
            }],
        };

        Ok(self
            .ctx
            .bot
            .interaction_client()
            .create_followup(&self.interaction.token)
            .payload_json(&serde_json::to_vec(&response)?)
            .await?)
    }
}
