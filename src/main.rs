use std::{collections::HashMap, env};

use serenity::{
    async_trait,
    model::{
        channel::ChannelType,
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandOptionType,
                ApplicationCommandPermissionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

mod db;
use db::Database;

mod challenges;
mod messages;

#[derive(Debug)]
pub enum InteractionError {
    /// Raised if a request is not in the format expected.
    // Note: Ideally, this would never happen because we have told Discord what format we expect
    // when registering the commands, supplying the modals, etc.
    UnprocessableRequest,

    /// Raised when a permissions check fails
    /// i.e. a user does not have the correct role, or roles could not be fetched because interaction was
    /// sent through a DM
    // Note: Ideally this would be caught by the built-in application commands permissions check, or indirectly
    // (like a response to a modal that could only be initiated by a restricted command).
    // But I don't trust it to be secure.
    Permissions,

    /// Error passed from underlying Discord API crate `serenity`
    Serenity(serenity::Error),

    /// Error with the database
    Db(db::DbError),

    /// Misc error
    Other(String),
}

impl From<db::DbError> for InteractionError {
    fn from(e: db::DbError) -> Self {
        Self::Db(e)
    }
}

impl From<serenity::Error> for InteractionError {
    fn from(e: serenity::Error) -> Self {
        Self::Serenity(e)
    }
}

pub type InteractionResult = Result<(), InteractionError>;

struct Handler {
    db: Database,
    admin_role_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(why) = match command.data.name.as_str() {
                    "submitflag" => challenges::cmd_submitflag(ctx, command).await,
                    "ping" => cmd_ping(ctx, command).await,
                    "addchallenge" => {
                        challenges::cmd_addchallenge(ctx, command, self.admin_role_id).await
                    }
                    "botmsg" => messages::cmd_botmsg(ctx, command, self.admin_role_id).await,
                    command_name => Err(InteractionError::Other(format!(
                        "Invalid command invoked: '{}'",
                        command_name
                    ))),
                } {
                    println!("Error when responding to application command: {:?}", why)
                }
            }
            Interaction::ModalSubmit(interaction) => {
                if let Err(why) = match interaction.data.custom_id.as_str() {
                    challenges::ID_MODAL_FLAG_SUBMIT => {
                        challenges::modal_submit_flag_response(ctx, &self.db, interaction).await
                    }
                    challenges::ID_MODAL_CHAL_ADD => {
                        challenges::modal_chal_add_response(
                            ctx,
                            &self.db,
                            interaction,
                            self.admin_role_id,
                        )
                        .await
                    }
                    messages::ID_MODAL_BOTMSG_SEND => {
                        messages::modal_botmsg_send_response(ctx, interaction, self.admin_role_id)
                            .await
                    }
                    messages::ID_MODAL_BOTMSG_EDIT => {
                        messages::modal_botmsg_edit_response(ctx, interaction, self.admin_role_id)
                            .await
                    }
                    modal_id => Err(InteractionError::Other(format!(
                        "Invalid id in modal submission: {:?}",
                        modal_id
                    ))),
                } {
                    println!("Error when responding to modal submission: {:?}", why);
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command
                        .name("submitflag")
                        .description("Invoke this command to submit a flag!")
                })
                .create_application_command(|command| {
                    command
                        .name("addchallenge")
                        .description("ROOT ONLY: add a challenge")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("botmsg")
                        .description("ROOT ONLY: Send and edit bot-authored messages")
                        .default_permission(false)
                        .create_option(|option| {
                            option
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .name("send")
                                .description("ROOT ONLY: Send a bot-authored message")
                                .create_sub_option(|option| {
                                    option
                                        .name("channel")
                                        .description("The destination channel for the message")
                                        .required(true)
                                        .kind(ApplicationCommandOptionType::Channel)
                                        .channel_types(&[ChannelType::Text])
                                })
                        })
                        .create_option(|option| {
                            option
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .name("edit")
                                .description("ROOT ONLY: Edit a bot-authored message")
                                .create_sub_option(|option| {
                                    option
                                        .name("channel")
                                        .description(
                                            "The channel containing the bot-authored message",
                                        )
                                        .required(true)
                                        .kind(ApplicationCommandOptionType::Channel)
                                        .channel_types(&[ChannelType::Text])
                                })
                                .create_sub_option(|option| {
                                    option
                                        .name("msgid")
                                        .description("The id of the bot-authored message to edit")
                                        .required(true)
                                        .kind(ApplicationCommandOptionType::String)
                                        .channel_types(&[ChannelType::Text])
                                })
                        })
                })
        })
        .await
        .unwrap();

        let command_id_map: HashMap<String, u64> = commands
            .iter()
            .map(|cmd| (cmd.name.clone(), cmd.id.0))
            .collect();

        let addchallenge_id = *command_id_map.get("addchallenge").unwrap();
        let botmsg_id = *command_id_map.get("botmsg").unwrap();

        let _perms =
            GuildId::set_application_commands_permissions(&guild_id, &ctx.http, |permissions| {
                permissions
                    .create_application_command(|command| {
                        command.id(addchallenge_id).create_permissions(|perm| {
                            perm.kind(ApplicationCommandPermissionType::Role)
                                .id(self.admin_role_id)
                                .permission(true)
                        })
                    })
                    .create_application_command(|command| {
                        command.id(botmsg_id).create_permissions(|perm| {
                            perm.kind(ApplicationCommandPermissionType::Role)
                                .id(self.admin_role_id)
                                .permission(true)
                        })
                    })
            })
            .await
            .unwrap();
    }
}

async fn cmd_ping(ctx: Context, command: ApplicationCommandInteraction) -> InteractionResult {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("pong!"))
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let admin_role_id: u64 = env::var("ADMIN_ROLE_ID")
        .expect("Expected ADMIN_ROLE_ID in environment")
        .parse()
        .expect("ADMIN_ROLE_ID must be an integer.");

    let sqlite_db_path = env::var("SQLITE_DB").expect("Expected SQLITE_DB in environment");

    let mut client = Client::builder(token)
        .event_handler(Handler {
            db: Database::new(&sqlite_db_path),
            admin_role_id,
        })
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
