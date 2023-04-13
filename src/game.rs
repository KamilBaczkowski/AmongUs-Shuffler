use std::collections::HashMap;

use serenity::{model::{prelude::{UserId, ChannelId}}, prelude::{TypeMapKey, RwLock}};

#[derive(Default)]
pub struct Games;

impl TypeMapKey for Games {
    type Value = RwLock<HashMap<UserId, Game>>;
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Game {
    owner: UserId,
    channel: ChannelId,
    pairs: Pairs,
}

impl Game {
    pub fn get_owner(&self) -> UserId {
        self.owner
    }

    pub fn get_pairs(self) -> Pairs {
        self.pairs
    }

    pub fn get_channel(&self) -> ChannelId {
        self.channel
    }
}

pub fn new_game(owner: UserId, channel: ChannelId, pairs: Pairs) -> Game {
    Game {
        owner,
        channel,
        pairs,
    }
}

pub type Players = Vec<UserId>;
pub type Pairs = Vec<(UserId, UserId)>;
