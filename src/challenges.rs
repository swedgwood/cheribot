use serenity::{
    client::Context,
    model::{
        id::RoleId,
        interactions::{
            application_command::ApplicationCommandInteraction,
            message_component::{ActionRowComponent, InputTextStyle},
            modal::ModalSubmitInteraction,
            InteractionResponseType,
        },
    },
    prelude::Mentionable,
};

use crate::{
    db::{
        models::{self, Challenge},
        Database,
    },
    InteractionError, InteractionResult,
};

pub const ID_MODAL_FLAG_SUBMIT: &str = "modal_flag_submit";
pub const ID_INPUT_MODAL_FLAG_SUBMIT: &str = "modal_flag_submit_input";

pub const ID_MODAL_CHAL_ADD: &str = "modal_chal_add";
pub const ID_INPUT_CHAL_MODAL_CHAL_ADD: &str = "modal_chal_add_input_chal";
pub const ID_INPUT_FLAG_MODAL_CHAL_ADD: &str = "modal_chal_add_input_flag";

pub async fn cmd_submitflag(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> InteractionResult {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|message| {
                    message
                        .custom_id(ID_MODAL_FLAG_SUBMIT)
                        .title("Submit flag")
                        .components(|components| {
                            components.create_action_row(|action| {
                                action.create_input_text(|input| {
                                    input
                                        .custom_id(ID_INPUT_MODAL_FLAG_SUBMIT)
                                        .style(InputTextStyle::Short)
                                        .label("Submit flag here: ")
                                        .required(true)
                                        .min_length(1)
                                        .max_length(100)
                                        .value("")
                                })
                            })
                        })
                })
        })
        .await?;

    Ok(())
}

pub async fn cmd_addchallenge(
    ctx: Context,
    command: ApplicationCommandInteraction,
    admin_role_id: u64,
) -> InteractionResult {
    if !command
        .member
        .as_ref()
        .ok_or(InteractionError::Permissions)?
        .roles
        .contains(&RoleId(admin_role_id))
    {
        return Err(InteractionError::Permissions);
    }

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|message| {
                    message
                        .custom_id(ID_MODAL_CHAL_ADD)
                        .title("Add a new challenge")
                        .components(|components| {
                            components
                                .create_action_row(|action| {
                                    action.create_input_text(|input| {
                                        input
                                            .custom_id(ID_INPUT_CHAL_MODAL_CHAL_ADD)
                                            .style(InputTextStyle::Short)
                                            .label("Challenge name:")
                                            .placeholder("Challenge name")
                                            .required(true)
                                            .min_length(1)
                                            .max_length(100)
                                            .value("")
                                    })
                                })
                                .create_action_row(|action| {
                                    action.create_input_text(|input| {
                                        input
                                            .custom_id(ID_INPUT_FLAG_MODAL_CHAL_ADD)
                                            .style(InputTextStyle::Short)
                                            .label("Flag:")
                                            .required(true)
                                            .min_length(1)
                                            .max_length(100)
                                            .value("")
                                    })
                                })
                        })
                })
        })
        .await?;

    Ok(())
}

pub async fn modal_chal_add_response(
    ctx: Context,
    db: &Database,
    interaction: ModalSubmitInteraction,
    admin_role_id: u64,
) -> InteractionResult {
    if !interaction
        .member
        .as_ref()
        .ok_or(InteractionError::Permissions)?
        .roles
        .contains(&RoleId(admin_role_id))
    {
        return Err(InteractionError::Permissions);
    }

    let name_component = interaction
        .data
        .components
        .get(0)
        .ok_or(InteractionError::UnprocessableRequest)?
        .components
        .get(0)
        .ok_or(InteractionError::UnprocessableRequest)?;

    let flag_component = interaction
        .data
        .components
        .get(1)
        .ok_or(InteractionError::UnprocessableRequest)?
        .components
        .get(0)
        .ok_or(InteractionError::UnprocessableRequest)?;

    if let (ActionRowComponent::InputText(name), ActionRowComponent::InputText(flag)) =
        (name_component, flag_component)
    {
        println!("Add chal: {:?}", (&name.value, &flag.value));

        Challenge::create_challenge(db, &name.value, &flag.value)?;

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|embed| embed.title("Added challenge successfully!"))
                    })
            })
            .await?;
    }

    Ok(())
}

pub async fn modal_submit_flag_response(
    ctx: Context,
    db: &Database,
    interaction: ModalSubmitInteraction,
) -> InteractionResult {
    if let ActionRowComponent::InputText(flag) = interaction
        .data
        .components
        .get(0)
        .ok_or(InteractionError::UnprocessableRequest)?
        .components
        .get(0)
        .ok_or(InteractionError::UnprocessableRequest)?
    {
        println!("Flag submitted! {}", flag.value);

        match models::Challenge::get_by_flag(db, &flag.value)? {
            Some(challenge) => {
                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data.embed(|embed| {
                                    embed
                                        .title("Flag correct!")
                                        .colour((0, 255, 0))
                                        .description(format!(
                                            "{} has scored the flag for challenge **{}**",
                                            interaction.user.mention(),
                                            challenge.name
                                        ))
                                })
                            })
                    })
                    .await?
            }
            None => {
                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data.embed(|embed| {
                                    embed
                                        .title("Flag incorrect!")
                                        .colour((255, 0, 0))
                                        .description("Incorrect flag!")
                                })
                            })
                    })
                    .await?
            }
        }

        Ok(())
    } else {
        Err(InteractionError::UnprocessableRequest)
    }
}
