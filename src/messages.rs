use serenity::{
    client::Context,
    model::{
        id::{ChannelId, RoleId},
        interactions::{
            application_command::ApplicationCommandInteraction,
            message_component::{ActionRowComponent, ButtonStyle, InputTextStyle},
            modal::ModalSubmitInteraction,
            InteractionResponseType,
        },
    },
};

use crate::{InteractionError, InteractionResult};

pub const ID_MODAL_BOTMSG_SEND: &str = "modal_botmsg_send";

pub const ID_INPUT_CONTENT_MODAL_BOTMSG_SEND: &str = "input_content_modal_botmsg_send";
pub const ID_INPUT_CHAN_MODAL_BOTMSG_SEND: &str = "input_chan_modal_botmsg_send";

pub const ID_MODAL_BOTMSG_EDIT: &str = "modal_botmsg_edit";

pub const ID_INPUT_MSG_MODAL_BOTMSG_EDIT: &str = "input_msg_modal_botmsg_edit";
pub const ID_INPUT_CHAN_MODAL_BOTMSG_EDIT: &str = "input_chan_modal_botmsg_edit";
pub const ID_INPUT_CONTENT_MODAL_BOTMSG_EDIT: &str = "input_content_modal_botmsg_edit";

pub async fn cmd_botmsg(
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

    match command.data.options.get(0).map(|o| o.name.as_str()) {
        Some("send") => cmd_botmsg_send(ctx, command).await,
        Some("edit") => cmd_botmsg_edit(ctx, command).await,
        _ => Err(InteractionError::UnprocessableRequest),
    }
}

async fn cmd_botmsg_send(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> InteractionResult {
    let channel_id = command
        .data
        .options
        .get(0)
        .and_then(|o| o.options.get(0))
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
        .ok_or(InteractionError::UnprocessableRequest)?;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|data| {
                    data.custom_id(ID_MODAL_BOTMSG_SEND)
                        .title("Send message as bot")
                        .components(|components| {
                            components
                                .create_action_row(|action_row| {
                                    action_row.create_input_text(|input_text| {
                                        input_text
                                            .custom_id(ID_INPUT_CONTENT_MODAL_BOTMSG_SEND)
                                            .label("Message content")
                                            .max_length(2000)
                                            .required(true)
                                            .style(InputTextStyle::Paragraph)
                                    })
                                })
                                .create_action_row(|action_row| {
                                    action_row.create_input_text(|input_text| {
                                        input_text
                                            .custom_id(ID_INPUT_CHAN_MODAL_BOTMSG_SEND)
                                            .label("Channel Id")
                                            .value(channel_id)
                                            .max_length(20)
                                            .required(true)
                                            .style(InputTextStyle::Short)
                                    })
                                })
                        })
                })
        })
        .await?;

    Ok(())
}

pub async fn modal_botmsg_send_response(
    ctx: Context,
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

    let msg_content = interaction
        .data
        .components
        .get(0)
        .and_then(|r| r.components.get(0))
        .and_then(|c| match c {
            ActionRowComponent::InputText(input_text) => Some(&input_text.value),
            _ => None,
        })
        .ok_or(InteractionError::UnprocessableRequest)?;

    let channel_id: u64 = interaction
        .data
        .components
        .get(1)
        .and_then(|r| r.components.get(0))
        .and_then(|c| match c {
            ActionRowComponent::InputText(input_text) => Some(&input_text.value),
            _ => None,
        })
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(InteractionError::UnprocessableRequest)?;

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    match ChannelId(channel_id)
        .send_message(&ctx.http, |message| message.content(msg_content))
        .await
    {
        Ok(sent_msg) => {
            interaction
                .create_followup_message(&ctx.http, |msg| {
                    msg.content("Bot-authored message sent!")
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                action_row.create_button(|button| {
                                    button
                                        .style(ButtonStyle::Link)
                                        .label("Go to message")
                                        .url(sent_msg.link())
                                })
                            })
                        })
                })
                .await?
        }
        Err(_) => {
            interaction
                .create_followup_message(&ctx.http, |msg| {
                    msg.content("Failed to send bot-authored message!")
                })
                .await?
        }
    };

    Ok(())
}

async fn cmd_botmsg_edit(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> InteractionResult {
    let channel_id = command
        .data
        .options
        .get(0)
        .and_then(|o| o.options.get(0))
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(InteractionError::UnprocessableRequest)?;

    let msg_id = command
        .data
        .options
        .get(0)
        .and_then(|o| o.options.get(1))
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(InteractionError::UnprocessableRequest)?;

    match ChannelId(channel_id).message(&ctx.http, msg_id).await {
        Ok(msg) => {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::Modal)
                        .interaction_response_data(|data| {
                            data.custom_id(ID_MODAL_BOTMSG_EDIT)
                                .title("Edit bot-authored message")
                                .components(|components| {
                                    components
                                        .create_action_row(|action_row| {
                                            action_row.create_input_text(|input_text| {
                                                input_text
                                                    .custom_id(ID_INPUT_CONTENT_MODAL_BOTMSG_EDIT)
                                                    .label("Message content")
                                                    .max_length(2000)
                                                    .value(msg.content)
                                                    .required(true)
                                                    .style(InputTextStyle::Paragraph)
                                            })
                                        })
                                        .create_action_row(|action_row| {
                                            action_row.create_input_text(|input_text| {
                                                input_text
                                                    .custom_id(ID_INPUT_CHAN_MODAL_BOTMSG_EDIT)
                                                    .label("Channel Id")
                                                    .value(channel_id)
                                                    .max_length(20)
                                                    .required(true)
                                                    .style(InputTextStyle::Short)
                                            })
                                        })
                                        .create_action_row(|action_row| {
                                            action_row.create_input_text(|input_text| {
                                                input_text
                                                    .custom_id(ID_INPUT_MSG_MODAL_BOTMSG_EDIT)
                                                    .label("Message Id")
                                                    .value(msg_id)
                                                    .max_length(20)
                                                    .required(true)
                                                    .style(InputTextStyle::Short)
                                            })
                                        })
                                })
                        })
                })
                .await?
        }
        Err(_) => {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|data| data.content("Failed to fetch message!"))
                })
                .await?
        }
    };

    Ok(())
}

pub async fn modal_botmsg_edit_response(
    ctx: Context,
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

    let msg_content = interaction
        .data
        .components
        .get(0)
        .and_then(|r| r.components.get(0))
        .and_then(|c| match c {
            ActionRowComponent::InputText(input_text) => Some(&input_text.value),
            _ => None,
        })
        .ok_or(InteractionError::UnprocessableRequest)?;

    let channel_id: u64 = interaction
        .data
        .components
        .get(1)
        .and_then(|r| r.components.get(0))
        .and_then(|c| match c {
            ActionRowComponent::InputText(input_text) => Some(&input_text.value),
            _ => None,
        })
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(InteractionError::UnprocessableRequest)?;

    let msg_id: u64 = interaction
        .data
        .components
        .get(2)
        .and_then(|r| r.components.get(0))
        .and_then(|c| match c {
            ActionRowComponent::InputText(input_text) => Some(&input_text.value),
            _ => None,
        })
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(InteractionError::UnprocessableRequest)?;

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    match ChannelId(channel_id)
        .edit_message(&ctx.http, msg_id, |msg| msg.content(msg_content))
        .await
    {
        Ok(edited_msg) => {
            interaction
                .create_followup_message(&ctx.http, |msg| {
                    msg.content("Bot-authored message edited!")
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                action_row.create_button(|button| {
                                    button
                                        .style(ButtonStyle::Link)
                                        .label("Go to message")
                                        .url(edited_msg.link())
                                })
                            })
                        })
                })
                .await?
        }
        Err(_) => {
            interaction
                .create_followup_message(&ctx.http, |msg| {
                    msg.content("Failed edit bot-authored message!")
                })
                .await?
        }
    };

    Ok(())
}
