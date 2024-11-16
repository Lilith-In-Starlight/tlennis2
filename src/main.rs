#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
mod game;
mod player;
mod team;

use game::{Game, Result, Run};
use rand::{prelude::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::{collections::HashMap, time::Duration};

use player::{Player, PlayerId};
use team::{Team, TeamId};

struct NameGenerator {
    names: Vec<String>,
    last_names: Vec<String>,
}

impl NameGenerator {
    fn generate<R: Rng>(&self, rng: &mut R) -> String {
        let name = self.names.choose(rng).unwrap();
        let last_name = self.last_names.choose(rng).unwrap();

        format!("{name} {last_name}")
    }
}

struct Data {
    teams: HashMap<TeamId, Team>,
    players: HashMap<PlayerId, Player>,
    name_generator: NameGenerator,
}

impl Data {
    pub fn get_team(&self, id: &TeamId) -> Option<&Team> {
        self.teams.get(id)
    }
    pub fn get_team_mut(&mut self, id: &TeamId) -> Option<&mut Team> {
        self.teams.get_mut(id)
    }
    pub fn get_player(&self, id: &PlayerId) -> Option<&Player> {
        self.players.get(id)
    }
    pub fn get_player_mut(&mut self, id: &PlayerId) -> Option<&mut Player> {
        self.players.get_mut(id)
    }

    pub fn new_player<R: Rng>(&mut self, rng: &mut R) -> PlayerId {
        let (id, player) = Player::new(&self.name_generator, rng);

        self.players.insert(id, player);

        id
    }

    pub fn add_team<R: Rng>(&mut self, name: String, rng: &mut R) -> TeamId {
        let team_key = TeamId::new();
        let new_team = Team::random_team(name, self, rng);
        self.teams.insert(team_key, new_team);

        team_key
    }
}

fn main() {
    let mut rng = ChaCha20Rng::from_entropy();
    let names = include_str!("../../data/names.txt");
    let names = names.lines().map(ToOwned::to_owned).collect();
    let last_names = include_str!("../../data/lastnames.txt");
    let last_names = last_names.lines().map(ToOwned::to_owned).collect();

    let name_generator = NameGenerator { names, last_names };

    let mut data = Data {
        teams: HashMap::new(),
        players: HashMap::new(),
        name_generator,
    };

    let home = data.add_team("The Speedles".to_owned(), &mut rng);
    let away = data.add_team("The Spabbles".to_owned(), &mut rng);

    let (mut game, mut result) = (Game::new(home, away, rng.gen()), Result::Continue);

    println!("{rng:#?}");

    loop {
        if matches!(result, Result::Finished) {
            break;
        }

        result = game.tick(&mut data, &mut rng);

        while let Some(report) = game.pop_report() {
            println!("{}", report.get_text(&data));
            std::thread::sleep(Duration::from_millis(
                (100 + 100 * report.comment.len()).try_into().unwrap(),
            ));
        }
    }
}
