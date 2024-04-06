#![feature(impl_trait_in_fn_trait_return)]

use std::sync::Arc;

use anyhow::Result;
use dotenvy_macro::dotenv;
use process_event::process_event;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::Level;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Config, ConfigBuilder, Shard, ShardId};
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::{
    application::interaction::Interaction,
    gateway::{
        payload::outgoing::update_presence::UpdatePresencePayload,
        presence::{self, ActivityType, MinimalActivity},
        Intents,
    },
};

mod commands;
mod process_event;

const TOKEN: &str = dotenv!("TOKEN");
const DATABASE_URL: &str = dotenv!("DATABASE_URL");

#[derive(Debug)]
struct App {
    http: HttpClient,
    db_pool: PgPool,
    cache: InMemoryCache,
}

impl App {
    async fn new(token: &str, database_url: &str) -> Result<Arc<Self>> {
        let app = Arc::new(App {
            http: HttpClient::new(token.to_string()),

            db_pool: PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?,

            cache: InMemoryCache::builder()
                .resource_types(ResourceType::MESSAGE)
                .build(),
        });

        Ok(app)
    }
}

trait MakeInteractionClient {
    fn interaction_client(
        &self,
        interaction: &Interaction,
    ) -> InteractionClient;
}

impl MakeInteractionClient for Arc<App> {
    fn interaction_client(
        &self,
        interaction: &Interaction,
    ) -> InteractionClient {
        self.http
            .interaction(interaction.application_id)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Init http client
    let app = App::new(TOKEN, DATABASE_URL).await?;

    // Create a shard
    let mut shard = Shard::with_config(ShardId::ONE, shard_config());

    // Register global commands
    commands::register_commands(app.clone()).await?;

    tracing::info!("logged in");

    // Event loop
    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "error receiving event");

                if source.is_fatal() {
                    tracing::error!(?source, "fatal error receiving event");
                    break;
                }

                continue;
            }
        };

        app.cache.update(&event);

        // Process event in new task
        tokio::spawn(process_event(event, app.clone()));
    }

    Ok(())
}

fn shard_config() -> Config {
    ConfigBuilder::new(TOKEN.to_string(), Intents::all())
        .presence(presence())
        .build()
}

fn presence() -> UpdatePresencePayload {
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("you"),
        url: None,
    };

    UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: presence::Status::Online,
    }
}
