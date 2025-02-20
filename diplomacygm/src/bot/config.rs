use std::{collections::HashSet, sync::LazyLock};

use serenity::futures::lock::Mutex;

fn is_member(string: &str, group: &[&str]) -> bool {
    group.contains(&string)
}

// Discord roles which are allowed full access to bot commands
const GM_ROLES: [&str; 4] = [
    "admin",
    "gm",
    "heavenly angel",
    "emergency gm",
];


pub fn is_gm_role(role: &str) -> bool {
    is_member(role, &GM_ROLES)
}

// Discord categories in which GM channels must be
// (so that you can't create a fake GM channel with the right name)
const GM_CATEGORIES: [&str; 1] = [
    "gm channels",
];

pub fn is_gm_category(category: &str) -> bool {
    is_member(category, &GM_CATEGORIES)
}


// Discord channels in which GMs are allowed to use non-public commands (e.g. adjudication)
const GM_CHANNELS: [&str; 1] = [
    "admin-chat",
];

pub fn is_gm_channel_name(channel: &str) -> bool {
    is_member(channel, &GM_CHANNELS)
}

// # Discord categories in which player channels must be
// # (so that you can't create a fake player channel with the right name)
// _player_categories: set[str] = {
//     "orders",
// }

// Discord categories in which GM channels must be
// (so that you can't create a fake GM channel with the right name)
const _PLAYER_CATEGORIES: [&str; 1] = [
    "orders",
];

pub fn _is_player_category(category: &str) -> bool {
    is_member(category, &_PLAYER_CATEGORIES)
}

// # Channel suffix for player orders channels.
// # E.g. if the player is "france" and the suffix is "-orders", the channel is "france-orders"
// player_channel_suffix: str = "-orders"

// Temporary bumbleship holds until the server restarts or until you fish too much
static TEMPORARY_BUMBLES: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn is_bumble(name: &str) -> bool {
    return name == "_bumble" || TEMPORARY_BUMBLES.lock().await.contains(name)
}

pub async fn add_temporary_bumble(name: &str) {
    TEMPORARY_BUMBLES.lock().await.insert(name.to_string());
}

pub async fn remove_temporary_bumble(name: &str) {
    TEMPORARY_BUMBLES.lock().await.remove(name);
}