// import logging
// import os
// import re
// import time
// from typing import Callable
// import inspect

// import discord
// from discord import HTTPException
// from discord.ext import commands

// from bot import command
// from diplomacy.persistence.manager import Manager

// intents = discord.Intents.default()
// intents.message_content = True
// bot = commands.Bot(command_prefix=".", intents=intents)
// logger = logging.getLogger(__name__)

// manager = Manager()

use core::option::Option::None;
use std::sync::Arc;

use poise::samples::HelpConfiguration;
use rand::seq::{IndexedRandom, SliceRandom};
use serenity::{all::{CreateMessage, GatewayIntents, Mention, ReactionType}, Client};
use tracing::info;

use crate::{bot::{config::{add_temporary_bumble, remove_temporary_bumble}, utils::reply_if_slash}, diplomacy::persistence::manager::Manager};

use super::{config::is_bumble, utils::{react, unreact}};

pub struct Data {
    pub manager: Manager,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type PrefixContext<'a> = poise::PrefixContext<'a, Data, Error>;

// TODO 
// # async def _handle_command(
// #     function: Callable[[commands.Context, Manager], tuple[str, str | None]],
// #     ctx: discord.ext.commands.Context,
// # ) -> None:
// #     start = time.time()

// #     ...

// #     elapsed = time.time() - start
// #     logger.debug(
// #         f"[{ctx.guild.name}][#{ctx.channel.name}]({ctx.message.author.name}) - '{ctx.message.content}' -> \n{response} | {elapsed}s"
// #     )

#[poise::command(prefix_command, slash_command, track_edits, category = "Utility")]
async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[rest]
    mut command: Option<String>,
) -> Result<(), Error> {
    // This makes it possible to just make `help` a subcommand of any command
    // `/fruit help` turns into `/help fruit`
    // `/fruit help apple` turns into `/help fruit apple`
    if ctx.invoked_command_name() != "help" {
        command = match command {
            Some(c) => Some(format!("{} {}", ctx.invoked_command_name(), c)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }
    let extra_text_at_bottom = "\
Type `.help command` for more info on a command.
You can edit your `.help` message to the bot and the bot will edit its response.";

    let config = HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// outputs information about a specific province
#[poise::command(prefix_command, slash_command, track_edits, category = "Utility")]
async fn province_info(
    ctx: Context<'_>,
    #[description = "Province or Coast name"]
    #[rest]
    name: String,
) -> Result<(), Error> {
    ctx.say(ctx.data().manager.province_info(ctx.guild_id().unwrap(), name)).await?;
    Ok(())
}


// def province_info(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     province_name = ctx.message.content.removeprefix(".province_info ").strip()
//     if not province_name:
//         raise ValueError("Usage: .province_info <province>")
//     province = board.get_location(province_name)
//     if province is None:
//         raise ValueError(f"Could not find province {province_name}")
//     # fmt: off
//     if isinstance(province, Province):
//         out = f"Province: {province.name}\n" + \
//             f"Type: {province.type.name}\n" + \
//             f"Coasts: {len(province.coasts)}\n" + \
//             f"Owner: {province.owner.name if province.owner else 'None'}\n" + \
//             f"Unit: {(province.unit.player.name + ' ' + province.unit.unit_type.name) if province.unit else 'None'}\n" + \
//             f"Center: {province.has_supply_center}\n" + \
//             f"Core: {province.core.name if province.core else 'None'}\n" + \
//             f"Half-Core: {province.half_core.name if province.half_core else 'None'}\n" + \
//             f"Adjacent Provinces:\n- " + "\n- ".join(sorted([adjacent.name for adjacent in province.adjacent])) + "\n"
//     else:
//         out = f"""Province: {province.name}
// Type: COAST
// Adjacent Provinces:
// - """ + "\n- ".join(sorted([adjacent.name for adjacent in province.adjacent_seas])) + "\n"
//     # fmt: on
//     return out, None

const PING_TEXT_CHOICES: [&str; 3] = ["proudly states", "fervently believes in the power of", "is being mind controlled by"];

/// Checks bot listens and responds.
#[poise::command(
    slash_command, 
    prefix_command,
    category = "Utility",
)]
async fn ping(
    ctx: Context<'_>,
    #[rest] text: Option<String>,
) -> Result<(), Error> {
    let mut response = String::from("Beep Boop");
    if rand::random::<f64>() < 0.1 {
        let author = ctx.author();
        let name = author.nick_in(ctx, ctx.guild_id().unwrap()).await.unwrap_or(author.name.to_string());
        let content = text.unwrap_or(String::from("nothing"));
        
        response = format!("{} {} {}", name, PING_TEXT_CHOICES.choose(&mut rand::rng()).unwrap(), content);        
    }

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    hide_in_help = true,
    check = "super::perms::gm_check",
)]
async fn botsay(ctx: Context<'_>, mention: Option<Mention>, #[rest] text: Option<String>) -> Result<(), Error> {
    if let Some(Mention::Channel(channel_id)) = mention {

        if let Some(content) = text {
            channel_id.send_message(ctx, CreateMessage::new().content(&content)).await?;

            info!("{} asked me to say '{}' in {}", ctx.author(), content, channel_id.name(ctx).await.unwrap())    
        }
    } else {
        ctx.say("No channel mention in message").await?;
        return Ok(());
    }

    reply_if_slash(ctx, "Sent message!").await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    hide_in_help = true,
)]
async fn bumble(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let mut list_of_bumble = ["b", "u", "m", "b", "l", "e"];
    list_of_bumble.shuffle(&mut rand::rng());
    let mut word_of_bumble = list_of_bumble.join("");

    let author_name = ctx.author().name.as_str();

    if is_bumble(author_name) && rand::random_range(0..10) == 0 {
        word_of_bumble = String::from("bumble");
    }

    match word_of_bumble.as_str() {
        "bumble" => {
            word_of_bumble = String::from("You are the chosen bumble");
            add_temporary_bumble(author_name);
        },
        "elbmub" => {
            word_of_bumble = String::from("elbmub nesohc eht era uoY");
        },
        _ => {}
    };

    ctx.data().manager.change_fish(ctx.guild_id().unwrap(), -1);

    ctx.say(word_of_bumble).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    hide_in_help = true,
)]
pub async fn fish(
    ctx: PrefixContext<'_>
) -> Result<(), Error> {
//  board = manager.get_board(ctx.guild.id)

    let mut fish_num = rand::random_range(0..20);
    let mut debumblify = false;
    let author_name = ctx.author().name.as_str();
    let is_bumble = is_bumble(author_name);

    if is_bumble && rand::random_range(0..10) == 0 {
        // Bumbles are good fishers
        if fish_num == 1 {
            fish_num = 0;
        }
        else if fish_num > 15 {
            fish_num -= 5;
        }
    }

    let mut fish_message;
    let fish_change;

    match fish_num {
        0 => {
            // something special
            let rare_fish_options = [
                ":dolphin:",
                ":shark:",
                ":duck:",
                ":goose:",
                ":dodo:",
                ":flamingo:",
                ":penguin:",
                ":unicorn:",
                ":swan:",
                ":whale:",
                ":seal:",
                ":sheep:",
                ":sloth:",
                ":hippopotamus:",
            ];
            fish_change = 10;
            fish_message = format!("**Caught a rare fish!** {}", rare_fish_options.choose(&mut rand::rng()).unwrap());
        },
        1..16 => {
            fish_num = (fish_num + 1) / 2;
            fish_change = fish_num;
            let fish_emoji_options: [(&str, i32); 5] = [(":fish:", 8), (":tropical_fish:", 4), (":blowfish:", 2), (":jellyfish:", 1), (":shrimp:", 2)];
            fish_message = format!("Caught {} fish! ", fish_num);
            for _ in 0..fish_num {
                let fish =fish_emoji_options.choose_weighted(&mut rand::rng(), |item| item.1).unwrap().0;
                fish_message += fish;
            }
        },
        _ => {
            fish_num = (21 - fish_num) / 2;

            if is_bumble {
                if rand::random_range(0..20) == 0 {
                    // Sometimes Bumbles are so bad at fishing they debumblify
                    debumblify = true;
                    fish_num = rand::random_range(10..20);
                } else {
                    // Bumbles that lose fish lose a lot of them
                    fish_num *= rand::random_range(3..10);
                }
            }
    
            fish_change = -fish_num;
            let fish_kind = "captured"; // if board.fish >= 0 else "future"
            fish_message = format!("Accidentally let {} {} fish sneak away :(", fish_num, fish_kind);
        }
    }

    let total_fish = ctx.data.manager.change_fish(ctx.guild_id().unwrap(), fish_change);

    fish_message += &format!("\nIn total, {} fish have been caught!", total_fish);
//     if random.randrange(0, 5) == 0:
//         get_connection().execute_arbitrary_sql(
//             """UPDATE boards SET fish=? WHERE board_id=? AND phase=?""",
//             (board.fish, board.board_id, board.get_phase_and_year_string()),
//         )

    if debumblify {
        remove_temporary_bumble(author_name);
        fish_message = format!("Your luck has run out! {}\nBumble is sad, you must once again prove your worth by Bumbling!", fish_message);
    }

    ctx.say(fish_message).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    hide_in_help = true,
)]
async fn phish(
    ctx: PrefixContext<'_>
) -> Result<(), Error> {
    let mut message = "No! Phishing is bad!";

    if is_bumble(ctx.author().name.as_str()) {
        message = "Please provide your firstborn pet and your soul for a chance at winning your next game!";
    }

    ctx.say(message).await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    hide_in_help = true,
)]
async fn cheat(
    ctx: PrefixContext<'_>
) -> Result<(), Error> {
    let mut message = String::from("Cheating is disabled for this user.");
    let author_name = &ctx.author().name;
    let nick_name = ctx.author().nick_in(ctx, ctx.guild_id().unwrap()).await.unwrap_or(author_name.to_string());

    // TODO board = manager.get_board(ctx.guild.id)
    if is_bumble(author_name) {
        let random_player = "todo";
        let random_province = "todo";
        let sources = [
                format!("It looks like {} is getting coalitioned this turn :cry:", nick_name),
                format!("{} is talking about stabbing {} again", nick_name, random_player),
                format!("looks like he's throwing to {}... shame", nick_name),
                "yeah".to_string(),
                "People in this game are not voiding enough".to_string(),
                format!("I can't believe {} is moving to {}", nick_name, random_province),
                format!("{} has a bunch of invalid orders", nick_name),
                format!("No one noticed that {} overbuilt?", nick_name),
                format!("{} is in a perfect position to stab {}", random_player, nick_name),
                ".bumble".to_string(),
            ];
        let sample = sources.choose(&mut rand::rng()).unwrap();

        message = format!("Here\'s a helpful message I stole from the spectator chat: \n\"{}\"", sample);
    }

    ctx.say(message).await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    hide_in_help = true,
)]
async fn advice(
    ctx: PrefixContext<'_>
) -> Result<(), Error> {
    let mut message = "You are not worthy of advice.";

    if is_bumble(ctx.author().name.as_str()) {
        message = "Bumble suggests that you go fishing, although typically blasphemous, today is your lucky day!";
    } else if rand::random_range(0..5) == 0 {
        let advices = [
            "Bumble was surprised you asked him for advice and wasn't ready to give you any, maybe if you were a true follower...",
            "Icecream demands that you void more and will not be giving any advice until sated.",
            "Salt suggests that stabbing all of your neighbors is a good play in this particular situation.",
            "Ezio points you to an ancient proverb: see dot take dot.",
            "CaptainMeme advises balance of power play at this instance.",
            "Ash Lael deems you a sufficiently apt liar, go use those skills!",
            "Kwiksand suggests winning.",
            "Ambrosius advises taking the opportunity you've been considering, for more will ensue.",
            "The GMs suggest you input your orders so they don't need to hound you for them at the deadline.",
        ];
        message = advices.choose(&mut rand::rng()).unwrap();
    }

    ctx.say(message).await?;
    Ok(())
}

/// Create a game of Imp Dip and output the map.
/// 
/// Create a game of Imp Dip and output the map. (there are no other variant options at this time)
#[poise::command(
    prefix_command, slash_command,
    category = "GM",
    check = "super::perms::gm_check",
)]
async fn create_game(
    ctx: Context<'_>,
    #[description = "Game type to create; None for impdip"]
    #[rest]
    command: Option<String>,

) -> Result<(), Error> {
    let gametype = match command {
        Some(game) => game,
        None => String::from("impdip1.1.json"),
    };

    let manager = &ctx.data().manager;
    let guild_id = ctx.guild_id().unwrap();

    let game_message = manager.create_game(&guild_id, gametype).await;

    ctx.reply(game_message).await?;
    Ok(())
}

// # @bot.command(hidden=True)
// # async def announce(ctx: discord.ext.commands.Context) -> None:
// #     await command.announce(ctx, {bot.get_guild(server_id) for server_id in manager.list_servers()})


// # @bot.command(
// #     brief="Submits orders; there must be one and only one order per line.",
// #     description="""Submits orders: 
// #     There must be one and only one order per line.
// #     A variety of keywords are supported: e.g. '-', '->', 'move', and 'm' are all supported for a move command.
// #     Supplying the unit type is fine but not required: e.g. 'A Ghent -> Normandy' and 'Ghent -> Normandy' are the same
// #     If anything in the command errors, we recommend resubmitting the whole order message.
// #     *During Build phases only*, you have to specify multi-word provinces with underscores; e.g. Somali Basin would be Somali_Basin (we use a different parser during build phases)
// #     If you would like to use something that is not currently supported please inform your GM and we can add it.""",
// # )
// # async def order(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.order, ctx)


// # @bot.command(
// #     brief="Removes orders for given units.",
// #     description="Removes orders for given units (required for removing builds/disbands). "
// #     "There must be one and only one order per line.",
// # )
// # async def remove_order(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.remove_order, ctx)


// # @bot.command(
// #     brief="Outputs your current submitted orders.",
// #     description="Outputs your current submitted orders. "
// #     "In the future we will support outputting a sample moves map of your orders.",
// # )
// # async def view_orders(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.view_orders, ctx)


// # @bot.command(brief="Adjudicates the game and outputs the moves and results maps.")
// # async def adjudicate(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.adjudicate, ctx)


// # @bot.command(brief="Rolls back to the previous game state.")
// # async def rollback(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.rollback, ctx)


// # @bot.command(brief="Reloads the current board with what is in the DB")
// # async def reload(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.reload, ctx)


// # @bot.command(brief="Outputs the scoreboard.", description="Outputs the scoreboard.")
// # async def scoreboard(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.get_scoreboard, ctx)


// # @bot.command(
// #     brief="Edits the game state and outputs the results map.",
// #     description="""Edits the game state and outputs the results map. 
// #     There must be one and only one command per line.
// #     Note: you cannot edit immalleable map state (eg. province adjacency).
// #     The following are the supported sub-commands:
// #     * set_phase {spring, fall, winter}_{moves, retreats, builds}
// #     * set_core <province_name> <player_name>
// #     * set_half_core <province_name> <player_name>
// #     * set_province_owner <province_name> <player_name>
// #     * create_unit {A, F} <player_name> <province_name>
// #     * create_dislodged_unit {A, F} <player_name> <province_name> <retreat_option1> <retreat_option2>...
// #     * delete_unit <province_name>
// #     * move_unit <province_name> <province_name>
// #     * dislodge_unit <province_name> <retreat_option1> <retreat_option2>...
// #     * make_units_claim_provinces {True|(False) - whether or not to claim SCs}""",
// # )
// # async def edit(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.edit, ctx)


// # @bot.command(brief="Clears all players orders.")
// # async def remove_all(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.remove_all, ctx)

// # @bot.command(
// #     brief="disables orders until .unlock_orders is run.",
// #     description="""disables orders until .enable_orders is run.
// #              Note: Currently does not persist after the bot is restarted""",
// # )
// # async def lock_orders(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.disable_orders, ctx)


// # @bot.command(brief="re-enables orders")
// # async def unlock_orders(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.enable_orders, ctx)


// # @bot.command(brief="outputs information about the current game")
// # async def info(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.info, ctx)


// # @bot.command(brief="outputs information about a specific province")
// # async def province_info(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.province_info, ctx)


// # @bot.command(brief="outputs all provinces per owner")
// # async def all_province_data(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.all_province_data, ctx)


// # @bot.command(
// #     brief="archives a category of the server",
// #     description="Used after a game is done. Will make all channels in category viewable by all server members, but no messages allowed.)",
// # )
// # async def archive(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.archive, ctx)

// # @bot.command(
// #         brief="permanently deletes a game, cannot be undone"
// # )
// # async def delete_game(ctx: discord.ext.commands.Context) -> None:
// #     await _handle_command(command.delete_game, ctx)

// async def announce(ctx: commands.Context, servers: set[Guild | None]) -> None:
//     if not is_admin(ctx.message.author) and is_gm_channel(ctx.channel):
//         return
//     await ctx.message.add_reaction("👍")
//     content = ctx.message.content.removeprefix(".announce").strip()
//     logger.info(f"{ctx.message.author.name} sent announcement '{content}'")
//     for server in servers:
//         if server is None:
//             continue
//         admin_chat_channel = next(channel for channel in server.channels if is_gm_channel(channel))
//         if admin_chat_channel is None:
//             continue
//         await admin_chat_channel.send(f"__Announcement__\n{ctx.message.author.display_name} says:\n{content}")


// @perms.player("order")
// def order(player: Player | None, ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)

//     if player and not board.orders_enabled:
//         return "Orders locked! If you think this is an error, contact a GM.", None

//     return parse_order(ctx.message.content, player, board), None


// @perms.player("remove orders")
// def remove_order(player: Player | None, ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)

//     if player and not board.orders_enabled:
//         return "Orders locked! If you think this is an error, contact a GM.", None

//     return parse_remove_order(ctx.message.content, player, board), None


// @perms.player("view orders")
// def view_orders(player: Player | None, ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     try:
//         order_text = get_orders(manager.get_board(ctx.guild.id), player)
//     except RuntimeError as err:
//         logger.error(f"View_orders text failed in game with id: {ctx.guild.id}", exc_info=err)
//         order_text = "view_orders text failed"
//     if player is None:
//         try:
//             file_name = manager.draw_moves_map(ctx.guild.id, None)
//         except Exception as err:
//             logger.error(f"View_orders map failed in game with id: {ctx.guild.id}", exc_info=err)
//             file_name = None
//         return order_text, file_name

//     else:
//         # file_name = manager.draw_moves_map(ctx.guild.id, player)
//         return order_text, None


// @perms.gm("adjudicate")
// def adjudicate(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     svg_file_name = manager.adjudicate(ctx.guild.id)
//     return "Adjudication completed successfully.", svg_file_name


// @perms.gm("rollback")
// def rollback(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     return manager.rollback(ctx.guild.id)


// @perms.gm("reload")
// def reload(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     return manager.reload(ctx.guild.id)


// @perms.gm("remove all orders")
// def remove_all(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     for unit in board.units:
//         unit.order = None

//     database = get_connection()
//     database.save_order_for_units(board, board.units)
//     return "Successful", None


// # @perms.gm("get scoreboard")
// def get_scoreboard(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     response = ""
//     for player in board.get_players_by_score():
//         response += f"\n__{player.name}__: {len(player.centers)} ({'+' if len(player.centers) - len(player.units) >= 0 else ''}{len(player.centers) - len(player.units)})"
//     return response, None


// @perms.gm("edit")
// def edit(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     return parse_edit_state(ctx.message.content, manager.get_board(ctx.guild.id))



// @perms.gm("unlock orders")
// def enable_orders(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     board.orders_enabled = True
//     return "Successful", None


// @perms.gm("lock orders")
// def disable_orders(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     board.orders_enabled = False
//     return "Successful", None

// @perms.gm("delete the game")
// def delete_game(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     manager.total_delete(ctx.guild.id)
//     return "Game deleted", None

// def info(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)
//     out = "Phase: " + str(board.phase) + "\nOrders are: " + ("Open" if board.orders_enabled else "Locked")
//     return out, None


// def all_province_data(ctx: commands.Context, manager: Manager) -> tuple[str, str | None]:
//     board = manager.get_board(ctx.guild.id)

//     province_by_owner = defaultdict(list)
//     for province in board.provinces:
//         owner = province.owner
//         if not owner:
//             owner = "None"
//         province_by_owner[owner].append(province.name)

//     data = ""
//     for owner, provinces in province_by_owner.items():
//         data += f"{owner}: "
//         for province in provinces:
//             data += f"{province}, "
//         data += "\n\n"

//     return data, None


// # needed due to async
// from bot.utils import is_gm, is_gm_channel


// async def archive(ctx: commands.Context, _: Manager) -> tuple[str, str | None]:

//     if not is_gm(ctx.message.author):
//         raise PermissionError(f"You cannot archive because you are not a GM.")

//     if not is_gm_channel(ctx.channel):
//         raise PermissionError(f"You cannot archive in a non-GM channel.")

//     categories = [channel.category for channel in ctx.message.channel_mentions]
//     if not categories:
//         return "This channel is not part of a category.", None

//     for category in categories:
//         for channel in category.channels:
//             overwrites = channel.overwrites

//             # Remove all permissions except for everyone
//             overwrites.clear()
//             overwrites[ctx.guild.default_role] = PermissionOverwrite(read_messages=True, send_messages=False)

//             # Apply the updated overwrites
//             await channel.edit(overwrites=overwrites)

//     return f"The following catagories have been archived: {' '.join([catagory.name for catagory in categories])}", None

async fn pre_command(ctx: Context<'_>) {
    // TODO logger.debug(f"[{ctx.guild.name}][#{ctx.channel.name}]({ctx.message.author.name}) - '{ctx.message.content}'")

    // # mark the message as seen
    if let Context::Prefix(c) = ctx {
        let name = ctx.invoked_command_name();
        if name == "fish" || name == "phish" {
            c.msg.react(ctx, ReactionType::Unicode(String::from("🐟"))).await.expect("Failed to React");
        } else {
            c.msg.react(ctx, ReactionType::Unicode(String::from("👍"))).await.expect("Failed to React");
        }
    };
}

pub async fn run_bot(token: String) {
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let manager = Manager::new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600)))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            pre_command: |ctx| Box::pin(pre_command(ctx)),
            post_command: |_ctx| {
                Box::pin(async move {
                    // TODO logger.debug(f"[{ctx.guild.name}][#{ctx.channel.name}]({ctx.message.author.name}) - '{ctx.message.content}'")
                })
            },
            command_check: Some(|_ctx| {
                Box::pin(async move {
                    // TODO pass invocation data
                    Ok(true)
                })
            }),
            on_error: |error| {
                Box::pin(async move {
                    if let Some(Context::Prefix(c)) = error.ctx() {
                        let _ = react(c, '❌').await;
                        let _ = unreact(c, '👍').await;

                        c.say(error.to_string()).await.expect("Failed to display error");
                    };
                })
            },
            commands: vec![
                help(),
                ping(),
                botsay(),
                bumble(),
                fish(),
                phish(),
                cheat(),
                advice(),
                create_game(),
                province_info()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { manager })
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.
    let client =
        Client::builder(&token, intents).framework(framework).await;

    // Start listening for events by starting a single shard
    if let Err(why) = client.expect("Err creating client").start().await {
        println!("Client error: {why:?}");
    }
}