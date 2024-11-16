use rand::prelude::SliceRandom;
use rand::Rng;
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
        (data.get_team(self)).and_then(Team::get_current_player)
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

    pub fn random_team<R: Rng>(name: String, data: &mut Data, rng: &mut R) -> Self {
        let mut players = vec![];
        for _ in 0..=rng.gen_range(6..12) {
            players.push(data.new_player(rng));
        }

        Self {
            name,
            players,
            current_player: 0,
        }
    }
    pub fn shuffle_players<R: Rng>(&mut self, rng: &mut R) {
        self.players.shuffle(rng);
    }
}
