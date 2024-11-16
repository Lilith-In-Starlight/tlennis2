mod game;
mod player;
mod team;

use game::{Game, RunGame, Weather};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, time::Duration};

use player::{Player, PlayerId};
use team::{Team, TeamId};

struct NameGenerator {
    names: Vec<String>,
    last_names: Vec<String>,
}

impl NameGenerator {
    fn generate(&self) -> String {
        let name = self.names.choose(&mut rand::thread_rng()).unwrap();
        let last_name = self.last_names.choose(&mut rand::thread_rng()).unwrap();

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

    pub fn new_player(&mut self) -> PlayerId {
        let (id, player) = Player::new(&self.name_generator);

        self.players.insert(id, player);

        id
    }

    pub fn add_team(&mut self, name: String) -> TeamId {
        let team_key = TeamId::new();
        let new_team = Team::random_team(name, self);
        self.teams.insert(team_key, new_team);

        team_key
    }
}

fn main() {
    let names = include_str!("../../data/names.txt");
    let names = names.lines().map(|x| x.to_owned()).collect();
    let last_names = include_str!("../../data/lastnames.txt");
    let last_names = last_names.lines().map(|x| x.to_owned()).collect();

    let name_generator = NameGenerator { names, last_names };

    let mut data = Data {
        teams: HashMap::new(),
        players: HashMap::new(),
        name_generator,
    };

    let home = data.add_team("The Speedles".to_owned());
    let away = data.add_team("The Spabbles".to_owned());

    let mut game = Game::new(home, away, thread_rng().gen());

    loop {
        while let Some(report) = game.pop_report() {
            println!("{}", report.get_text(&data));
            std::thread::sleep(Duration::from_millis(1200));
        }
        game.tick(&mut data);
    }
}
