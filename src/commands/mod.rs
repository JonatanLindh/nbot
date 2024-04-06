use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::{bail, Context, Result};

use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use tracing::{info, trace};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{
    application_command::CommandData, Interaction,
};

use twilight_model::application::command::Command as CommandStruct;

use crate::App;

mod credits;
mod hello_command;

pub fn run_command<'a, T: Command + Send>(
    interaction: Interaction,
    cmd_data: CommandData,
    app: Arc<App>,
) -> BoxFuture<'a, Result<()>> {
    async move {
        trace!("Command: {}", cmd_data.name);

        T::parse_data(cmd_data)?
            ._run(app, interaction)
            .await
    }
    .boxed()
}

pub type CommandRunner<'a> =
    fn(Interaction, CommandData, Arc<App>) -> BoxFuture<'a, Result<()>>;

struct CommandRegistrar<'a> {
    name: &'static str,
    command: fn() -> CommandStruct,
    runner: CommandRunner<'a>,
}

impl<'a> CommandRegistrar<'a> {
    pub const fn new<T: Command + Send>() -> Self {
        Self {
            name: T::NAME,
            command: || T::create_command().into(),
            runner: run_command::<T>,
        }
    }
}

inventory::collect!(CommandRegistrar<'static>);

pub async fn register_commands(app: Arc<App>) -> Result<()> {
    let commands = inventory::iter::<CommandRegistrar>()
        .inspect(|a| info!("Registering command: {}", a.name))
        .map(|a| (a.command)())
        .collect::<Vec<_>>();

    let application = app
        .http
        .current_user_application()
        .await?
        .model()
        .await?;

    // Register commands
    if let Err(error) = app
        .http
        .interaction(application.id)
        .set_global_commands(&commands)
        .await
    {
        tracing::error!(?error, "failed to register commands");
        bail!("failed to register commands")
    }

    Ok(())
}

pub async fn process_command(
    interaction: Interaction,
    cmd_data: CommandData,
    app: Arc<App>,
) -> Result<()> {
    type CommandMap<'a> = HashMap<&'static str, CommandRunner<'a>>;

    static COMMAND_MAP: OnceLock<CommandMap> = OnceLock::new();

    COMMAND_MAP
        .get_or_init(|| {
            inventory::iter::<CommandRegistrar>()
                .map(|c| (c.name, c.runner))
                .collect::<CommandMap>()
        })
        .get(&*cmd_data.name)
        .with_context(|| format!("unknown command: {}", &*cmd_data.name))?(
        interaction,
        cmd_data,
        app,
    )
    .await
}

#[async_trait]
pub trait Command: CommandModel + CreateCommand {
    async fn _run(self, app: Arc<App>, interaction: Interaction) -> Result<()>;

    fn parse_data(cmd_data: CommandData) -> Result<Self> {
        Self::from_interaction(cmd_data.into())
            .context("Failed to parse command data")
    }
}
