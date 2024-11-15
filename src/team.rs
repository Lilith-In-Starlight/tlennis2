use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::{player::PlayerId, Data};

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct TeamId {
    uuid: Uuid,
}

pub struct Team {
    name: String,
    players: Vec<PlayerId>,
    current_player: usize,
}

impl TeamId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
    pub fn get_current_player(&self, data: &Data) -> Option<PlayerId> {
        (data.get_team(self)).and_then(|x| x.get_current_player())
    }

    pub fn get_current_player_mut<'a>(&self, data: &'a mut Data) -> Option<&'a mut PlayerId> {
        data.get_team_mut(self)
            .and_then(|x| x.get_current_player_mut())
    }
}

impl Team {
    pub fn get_current_player(&self) -> Option<PlayerId> {
        self.players.get(self.current_player).copied()
    }

    pub fn get_current_player_mut(&mut self) -> Option<&mut PlayerId> {
        self.players.get_mut(self.current_player)
    }

    pub fn random_team(name: String, data: &mut Data) -> Self {
        let mut players = vec![];
        for _ in 0..=thread_rng().gen_range(6..12) {
            players.push(data.new_player())
        }

        Self {
            name,
            players,
            current_player: 0,
        }
    }
    pub fn shuffle_players(&mut self) {
        self.players.shuffle(&mut thread_rng())
    }
}
