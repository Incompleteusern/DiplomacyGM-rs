// from discord.ext import commands

// from bot import config

// from diplomacy.persistence import phase
// from diplomacy.persistence.board import Board
// from diplomacy.persistence.manager import Manager
// from diplomacy.persistence.player import Player
// from diplomacy.persistence.unit import UnitType, Unit

// whitespace_dict = {
//     "_",
// }

// _north_coast = "nc"
// _south_coast = "sc"
// _east_coast = "ec"
// _west_coast = "wc"

// coast_dict = {
//     _north_coast: ["nc", "north coast", "(nc)"],
//     _south_coast: ["sc", "south coast", "(sc)"],
//     _east_coast: ["ec", "east coast", "(ec)"],
//     _west_coast: ["wc", "west coast", "(wc)"],
// }

// _army = "army"
// _fleet = "fleet"

// unit_dict = {
//     _army: ["a", "army", "cannon"],
//     _fleet: ["f", "fleet", "boat", "ship"],
// }

// _spring_moves = "spring moves"
// _spring_retreats = "spring retreats"
// _fall_moves = "fall moves"
// _fall_retreats = "fall retreats"
// _winter_builds = "winter builds"


// def is_admin(author: commands.Context.author) -> bool:
//     return author.name in ["eebopmasch", "icecream_guy", "_bumble"]

use tracing::info;

use super::{bot::{Context, Error, PrefixContext}, config::{is_gm_category, is_gm_channel_name, is_gm_role}};


use std::{fs::File, io::{self, Read, Write}, path::Path};

use poise::CreateReply;
use serenity::all::{CreateAttachment, ReactionType};
use zip::write::SimpleFileOptions;

pub fn sanitize_apostrophes(s: String) -> String {
    s.chars()
    .map(|x| match x { 
        '‘' | '’' | '`' | '´' | '′' | '‛' | '\'' => '\'', 
        _ => x
    }).collect()
}

// TODO elapsed time of a command
pub async fn send_response(mut response: String, file: Option<&Path>, ctx: Context<'_>) -> Result<(), Error> {

    while response.len() > 2000 {
        let cutoff = response[0..2000].rfind("\n").unwrap_or(2000);
        let (send, response_str) = response.split_at_mut(cutoff);

        ctx.say(send).await?;

        response = response_str.to_string();
    }

    let mut reply = CreateReply::default().content(response);

    if let Some(file_path) = file {
        let length = file_path.metadata().unwrap().len();

        // TODO don't use zip library just call cmd line
        if length > const { 10 * 1024 * 1024 } {
            let mut new_path = file_path.to_owned();
            new_path.set_file_name(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".zip");

            let zip_file = std::fs::File::create(&new_path)?;
            let mut zip = zip::ZipWriter::new(zip_file);
    
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                // files over u32::MAX require this flag set.
                .large_file(false);
            zip.start_file(file_path.file_name().unwrap().to_str().unwrap(), options)?;

            let mut buffer: Vec<u8> = Vec::new();
            io::copy(&mut File::open(file_path).unwrap().take(u64::MAX), &mut buffer)?;
    
            zip.write_all(&buffer)?;
            
            zip.finish()?;

            reply = reply.attachment(CreateAttachment::path(new_path.as_path()).await.expect("Failed to attach zip file"));
        } else {
            reply = reply.attachment(CreateAttachment::path(file_path).await.expect("Failed to attach file"));
        }
    }


    ctx.send(reply).await?;

    Ok(())
}

pub async fn reply_if_slash(ctx: Context<'_>, s: impl Into<String>) -> Result<(), Error> {
    if let Context::Application(_) = ctx {
        ctx.send(CreateReply::default().content(s).ephemeral(true)).await?;
    }

    Ok(())
}

pub async fn react(ctx: PrefixContext<'_>, emoji: char) -> Result<(), Error> {
    println!("reacting with {}", emoji);
    
    ctx.msg.react(ctx, ReactionType::Unicode(String::from(emoji))).await.expect("Failed to React");
    println!("done reacting with {}", emoji);

    Ok(())
}

pub async fn unreact(ctx: PrefixContext<'_>, emoji: char) -> Result<(), Error> {
    let user_id = ctx.cache().current_user().id;
    ctx.msg.delete_reaction(ctx, Some(user_id), ReactionType::Unicode(String::from(emoji))).await.expect("Failed to Unreact");

    Ok(())
}

pub async fn is_gm(ctx: Context<'_>) -> bool {
    let author = ctx.author_member().await.expect("Failed to find author of messages");
    let guild_roles = ctx.guild_id().unwrap().roles(ctx).await.unwrap();

    let author_roles = guild_roles.iter().filter(|(id, _)| author.roles.contains(id)).map(|(_, role)| role.clone());

    for role in author_roles {
        info!("{}", role.name.to_lowercase());
        if is_gm_role(&role.name) {
            return true;
        }
    }

    return false;
}

pub async fn is_gm_channel(ctx: Context<'_>) -> bool {
    let channel = ctx.guild_channel().await.unwrap();
    
    let parent_id = channel.parent_id;

    if let Some(parent_id) = parent_id {
        return is_gm_channel_name(channel.name()) && is_gm_category(parent_id.name(ctx).await.unwrap().as_str());
    }

    return false;
    
}

// def get_player_by_channel(channel: commands.Context.channel, manager: Manager, server_id: int) -> Player | None:
//     name = channel.name
//     if not name.endswith(config.player_channel_suffix) or not config.is_player_category(channel.category.name):
//         return None
//     name = name[: -(len(config.player_channel_suffix))]
//     return get_player_by_name(name, manager, server_id)


// def get_player_by_name(name: str, manager: Manager, server_id: int) -> Player | None:
//     for player in manager.get_board(server_id).players:
//         if player.name.lower() == name.strip().lower():
//             return player
//     return None


// def is_player_channel(player_role: str, channel: commands.Context.channel) -> bool:
//     player_channel = player_role.lower() + config.player_channel_suffix
//     return player_channel == channel.name and config.is_player_category(channel.category.name)


// def get_keywords(command: str) -> list[str]:
//     """Command is split by whitespace with '_' representing whitespace in a concept to be stuck in one word.
//     e.g. 'A New_York - Boston' becomes ['A', 'New York', '-', 'Boston']"""
//     keywords = command.split(" ")
//     for i in range(len(keywords)):
//         for j in range(len(keywords[i])):
//             if keywords[i][j] in whitespace_dict:
//                 keywords[i] = keywords[i][:j] + " " + keywords[i][j + 1 :]

//     for i in range(len(keywords)):
//         keywords[i] = _manage_coast_signature(keywords[i])

//     return keywords


// def _manage_coast_signature(keyword: str) -> str:
//     for coast_key, coast_val in coast_dict.items():
//         # we want to make sure this was a separate word like "zapotec ec" and not part of a word like "zapotec"
//         suffix = f" {coast_val}"
//         if keyword.endswith(suffix):
//             # remove the suffix
//             keyword = keyword[: len(keyword) - len(suffix)]
//             # replace the suffix with the one we expect
//             new_suffix = f" {coast_key}"
//             keyword += f" {new_suffix}"
//     return keyword


// def get_unit_type(command: str) -> UnitType | None:
//     command = command.strip()
//     if command in unit_dict[_army]:
//         return UnitType.ARMY
//     if command in unit_dict[_fleet]:
//         return UnitType.FLEET
//     return None


// def get_orders(board: Board, player_restriction: Player | None) -> str:
//     if phase.is_builds(board.phase):
//         response = "Received orders:"
//         for player in sorted(board.players, key=lambda sort_player: sort_player.name):
//             if not player_restriction or player == player_restriction:
//                 response += f"\n**{player.name}**: ({len(player.centers)}) ({'+' if len(player.centers) - len(player.units) >= 0 else ''}{len(player.centers) - len(player.units)})"
//                 for unit in player.build_orders:
//                     response += f"\n{unit}"
//         return response
//     else:

//         if player_restriction is None:
//             players = board.players
//         else:
//             players = {player_restriction}

//         response = ""

//         for player in sorted(players, key=lambda p: p.name):
//             if phase.is_retreats(board.phase):
//                 in_moves = lambda u: u == u.province.dislodged_unit
//             else:
//                 in_moves = lambda _: True
//             moving_units = [unit for unit in player.units if in_moves(unit)]
//             ordered = [unit for unit in moving_units if unit.order is not None]
//             missing = [unit for unit in moving_units if unit.order is None]

//             response += f"**{player.name}** ({len(ordered)}/{len(moving_units)})\n"
//             if missing:
//                 response += f"__Missing Orders:__\n"
//                 for unit in sorted(missing, key=lambda _unit: _unit.province.name):
//                     response += f"{unit}\n"
//             if ordered:
//                 response += f"__Submitted Orders:__\n"
//                 for unit in sorted(ordered, key=lambda _unit: _unit.province.name):
//                     response += f"{unit} {unit.order}\n"

//         return response
