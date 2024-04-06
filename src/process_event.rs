use std::{mem, sync::Arc};

use anyhow::Result;
use twilight_gateway::Event;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    channel::Message,
};

use crate::{commands::process_command, App};

pub async fn process_event(event: Event, app: Arc<App>) -> Result<()> {
    match event {
        Event::InteractionCreate(i) => process_interaction(i.0, app).await?,
        Event::MessageCreate(msg) => process_message_create(msg.0, app).await?,

        Event::Ready(_) => {
            println!("Shard is ready");
        }

        _ => {}
    }

    Ok(())
}

async fn process_interaction(
    mut interaction: Interaction,
    app: Arc<App>,
) -> Result<()> {
    match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(d)) => {
            process_command(interaction, *d, app).await
        }
        _ => {
            tracing::warn!("ignoring non-command interaction");
            Ok(())
        }
    }
}

async fn process_message_create(msg: Message, app: Arc<App>) -> Result<()> {
    match msg.content.as_str() {
        "!ping" => {
            app.http
                .create_message(msg.channel_id)
                .content("Pong!")?
                .await?;
        }

        "!ping2" => {
            app.http
                .create_message(msg.channel_id)
                .content("Pong2!")?
                .await?;
        }

        _ => {}
    };

    Ok(())
}
