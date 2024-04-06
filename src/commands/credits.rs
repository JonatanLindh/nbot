use std::sync::Arc;

use anyhow::{Context, Result};
use nbot_macros::{bot_command, super_command};
use twilight_interactions::command::ResolvedUser;
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    embed::EmbedBuilder, InteractionResponseDataBuilder,
};

use crate::{App, MakeInteractionClient};

use super::Command;

const STARTING_CREDITS: i64 = 10;

#[derive(Debug)]
struct Member {
    id: i64,
    credits: i64,
}

#[derive(Debug)]
#[super_command(name = "credits", desc = "Credit commands")]
pub enum CreditsCommand {
    #[command(name = "count")]
    CreditsCountCommand,
}

#[bot_command(
    name = "count",
    desc = "Get credit count",
    register = false
)]
pub struct CreditsCountCommand {
    /// Who?
    user: Option<ResolvedUser>,
}

#[async_trait::async_trait]
impl Command for CreditsCountCommand {
    async fn _run(self, app: Arc<App>, interaction: Interaction) -> Result<()> {
        let user_id = match self.user {
            Some(user) => Some(user.resolved.id),
            None => interaction.author_id(),
        };

        let embed = if let Some(id) = user_id {
            let member = get_user(&app, id.get() as i64).await;

            if let Some(m) = member {
                EmbedBuilder::new()
                    .description(format!("{}", m.credits))
                    .color(0x34EB55)
                    .validate()?
                    .build()
            } else {
                EmbedBuilder::new()
                    .description("User is not yet registered")
                    .color(0xEBB434)
                    .validate()?
                    .build()
            }
        } else {
            EmbedBuilder::new()
                .description("Could not get user_id, wacky error")
                .color(0xEB4034)
                .validate()?
                .build()
        };

        let response_data = InteractionResponseDataBuilder::new()
            .embeds([embed])
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

async fn get_user(app: &Arc<App>, user_id: impl Into<i64>) -> Option<Member> {
    sqlx::query_as!(
        Member,
        "SELECT * FROM member where id = $1",
        user_id.into()
    )
    .fetch_optional(&app.db_pool)
    .await
    .unwrap()
}

async fn insert_user(app: &Arc<App>, user_id: i64) -> Result<u64> {
    sqlx::query!(
            "INSERT INTO member(id, credits) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            user_id,
            STARTING_CREDITS
        )
        .execute(&app.db_pool)
        .await
        .map(|r| r.rows_affected())
        .context("Insert user")
}
