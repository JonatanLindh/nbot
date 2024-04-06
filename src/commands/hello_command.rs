use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use nbot_macros::bot_command;
use twilight_interactions::command::ResolvedUser;
use twilight_mention::Mention;
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{App, MakeInteractionClient};

use super::Command;

#[bot_command(
    name = "hello",
    desc = "Say hello to other members"
)]
pub struct HelloCommand {
    /// Message to send
    message: String,
    /// User to send the message to
    user: Option<ResolvedUser>,
}

#[async_trait]
impl Command for HelloCommand {
    async fn _run(self, app: Arc<App>, interaction: Interaction) -> Result<()> {
        let message = match self.user {
            Some(ResolvedUser { resolved: user, .. }) => {
                format!("Hello {}!\n{}", user.id.mention(), self.message)
            }
            _ => self.message,
        };

        let response_data = InteractionResponseDataBuilder::new()
            .content(message)
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response_data),
        };

        app.interaction_client(&interaction)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }
}
