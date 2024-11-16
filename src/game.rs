mod report;

use std::{collections::VecDeque, fmt::Display};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use report::Report;

use crate::{
    player::Player,
    team::{Team, TeamId},
    Data,
};

// I plan to support multiple kinds of games so i'm making this a struct
pub trait Run {
    fn tick<R: Rng>(&mut self, data: &mut Data, rng: &mut R) -> Result;

    fn report(&mut self, comment: String, data: &Data);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Space {
    First,
    Middle,
    Third,
}

impl Space {
    pub fn farthest<R: Rng>(self, rng: &mut R) -> Self {
        match self {
            Self::First => Self::Third,
            Self::Middle => loop {
                let x: Self = rng.gen();
                if x != self {
                    break x;
                }
            },
            Self::Third => Self::First,
        }
    }
}

impl Distribution<Space> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Space {
        match rng.gen_range::<usize, _>(0..=2) {
            0 => Space::First,
            1 => Space::Middle,
            2 => Space::Third,
            _ => unreachable!(),
        }
    }
}

enum GameState {
    Serving(Side),
    PreHit(Side),
    Hit(Side, WeatherResult),
    Score(Side),
}

#[derive(Clone, Copy)]
enum Side {
    Home,
    Away,
}

impl Side {
    pub const fn opposite(self) -> Self {
        match self {
            Self::Home => Self::Away,
            Self::Away => Self::Home,
        }
    }
}

pub struct Game {
    home: PlayerState,
    away: PlayerState,
    ball_direction: Space,

    state: GameState,

    reports: VecDeque<Report>,

    weather: Weather,
}

pub struct PlayerState {
    team: TeamId,
    space: Space,
    score: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    None,
    Feedback,
    Reverb,
    Observation,
    Omni,
    Unpredictable,
}

impl Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Clear"),
            Self::Feedback => write!(f, "Feedback"),
            Self::Reverb => write!(f, "Reverb"),
            Self::Observation => write!(f, "Observation"),
            Self::Unpredictable => write!(f, "???"),
            Self::Omni => write!(f, "All"),
        }
    }
}

impl Distribution<Weather> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weather {
        match rng.gen_range::<usize, _>(0..=4) {
            0 => Weather::None,
            1 => Weather::Feedback,
            2 => Weather::Reverb,
            3 => Weather::Observation,
            4 => Weather::Unpredictable,
            _ => unreachable!(),
        }
    }
}

impl Game {
    pub fn pop_report(&mut self) -> Option<Report> {
        self.reports.pop_back()
    }
    pub const fn new(home: TeamId, away: TeamId, weather: Weather) -> Self {
        Self {
            home: PlayerState {
                team: home,
                space: Space::Middle,
                score: 0,
            },
            away: PlayerState {
                team: away,
                space: Space::Middle,
                score: 0,
            },
            ball_direction: Space::Middle,
            state: GameState::Serving(Side::Home),
            reports: VecDeque::new(),
            weather,
        }
    }
    const fn get_team(&self, team: Side) -> &PlayerState {
        match team {
            Side::Home => &self.home,
            Side::Away => &self.away,
        }
    }
    fn get_team_mut(&mut self, team: Side) -> &mut PlayerState {
        match team {
            Side::Home => &mut self.home,
            Side::Away => &mut self.away,
        }
    }
}

pub enum Result {
    Continue,
    Finished,
}

#[allow(clippy::too_many_lines)]
// ignore the fact that this is a trait and not a impl on the Game itself. i want to support having multiple kinds of games eventually
impl Run for Game {
    fn tick<R: Rng>(&mut self, data: &mut Data, rng: &mut R) -> Result {
        macro_rules! report {
            ($text:literal) => {
                self.report(format!($text), data);
            };
            ($text:literal, $($player:ident),+) => {
                self.report(format!($text, $($player),+), data);
            };
        }
        match self.state {
            GameState::Serving(serving_side) => {
                let (serving_state, receiving_state) = match serving_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };
                serving_state.space = Space::Middle;
                receiving_state.space = Space::Middle;
                self.ball_direction = rng.gen();

                let serving_player_name = serving_state
                    .team
                    .get_current_player(data)
                    .and_then(|x| data.get_player(&x))
                    .map(Player::get_name)
                    .unwrap();

                report!("{} serves!", serving_player_name);

                self.state = GameState::PreHit(serving_side.opposite());
                Result::Continue
            }
            GameState::PreHit(hitting_side) => {
                let (hitter_state, _) = match hitting_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                let hitting_player = hitter_state.team.get_current_player(data).unwrap();

                if hitter_state.space == self.ball_direction {
                    if data
                        .get_player(&hitting_player)
                        .unwrap()
                        .distraction_check(rng)
                    {
                        hitter_state.space = rng.gen();
                    }
                } else if data.get_player(&hitting_player).unwrap().speed_check(rng) {
                    hitter_state.space = self.ball_direction;
                } else {
                    hitter_state.space = rng.gen();
                }

                let weather_result = self.weather.pre_hit(hitting_side, self, data, rng);

                self.state = GameState::Hit(hitting_side, weather_result);
                Result::Continue
            }
            GameState::Hit(hitting_side, weather_result) => {
                let (hitter_state, _) = match hitting_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                let hitting_player = hitter_state.team.get_current_player(data).unwrap();

                match weather_result {
                    WeatherResult::Prevent => {
                        let hitting_player_name =
                            data.get_player(&hitting_player).unwrap().get_name();
                        report!("{} doesn't manage to hit!", hitting_player_name);
                        self.get_team_mut(hitting_side.opposite()).score += 1;

                        self.state = GameState::Score(hitting_side.opposite());
                        Result::Continue
                    }
                    WeatherResult::Nothing => {
                        if hitter_state.space == self.ball_direction {
                            if data.get_player(&hitting_player).unwrap().control_check(rng) {
                                self.ball_direction = self.ball_direction.farthest(rng);
                            } else {
                                self.ball_direction = rng.gen();
                            }

                            let hitting_player_name =
                                data.get_player(&hitting_player).unwrap().get_name();

                            report!("{hitting_player_name} hits!");
                            self.state = GameState::PreHit(hitting_side.opposite());
                        } else {
                            let hitting_player_name =
                                data.get_player(&hitting_player).unwrap().get_name();

                            report!("{hitting_player_name} fails to hit it!");
                            self.get_team_mut(hitting_side.opposite()).score += 1;

                            self.state = GameState::Score(hitting_side.opposite());
                        }
                        Result::Continue
                    }
                }
            }
            GameState::Score(scoring_side) => {
                let (scorer_state, _) = match scoring_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                let scoring_player_name = scorer_state
                    .team
                    .get_current_player(data)
                    .and_then(|x| data.get_player(&x))
                    .unwrap()
                    .get_name();

                report!("{scoring_player_name} scores!");

                let (scorer_state, other_state) = match scoring_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                if scorer_state.score > 4
                    && (other_state.score < 4 || scorer_state.score > other_state.score + 1)
                {
                    report!("{scoring_player_name} wins!");
                    return Result::Finished;
                }
                self.state = GameState::Serving(scoring_side.opposite());
                Result::Continue
            }
        }
    }

    fn report(&mut self, comment: String, data: &Data) {
        let report = Report::take_snapshot(self, data).with_comment(comment);
        self.reports.push_front(report);
    }
}

impl Weather {
    fn weather_announce(self, hitter: Side, game: &mut Game, data: &Data) {
        match self {
            Self::None => game.report("It's a sunny day!".to_owned(), data),
            Self::Feedback => {
                game.report("The feedback gathers around the players.".to_owned(), data);
            }
            Self::Reverb => game.report("The ground tremors with reverb.".to_owned(), data),
            Self::Observation => game.report("The clouds reveal eyes in the sky.".to_owned(), data),
            Self::Omni => game.report("We're experiencing everything.".to_owned(), data),
            Self::Unpredictable => {
                game.report("We don't know what the sky is doing.".to_owned(), data);
            }
        }
    }
    fn pre_hit<R: Rng>(
        self,
        hitter: Side,
        game: &mut Game,
        data: &mut Data,
        rng: &mut R,
    ) -> WeatherResult {
        let (hitter_state, _) = match hitter {
            Side::Home => (&mut game.home, &mut game.away),
            Side::Away => (&mut game.away, &mut game.home),
        };
        match self {
            Self::None => WeatherResult::Nothing,
            Self::Feedback => {
                if rng.gen::<f64>() < 0.05 {
                    let away_id = game.away.team.get_current_player(data).unwrap();
                    let home_id = game.home.team.get_current_player(data).unwrap();
                    *game.home.team.get_current_player_mut(data).unwrap() = away_id;
                    *game.away.team.get_current_player_mut(data).unwrap() = home_id;
                    game.report(
                        format!(
                            "{} has been feedbacked with {}!",
                            data.get_player(&home_id).unwrap().get_name(),
                            data.get_player(&away_id).unwrap().get_name()
                        ),
                        data,
                    );
                }
                WeatherResult::Nothing
            }
            Self::Reverb => {
                if rng.gen::<f64>() < 0.05 {
                    data.get_team_mut(&game.home.team)
                        .unwrap()
                        .shuffle_players(rng);
                    data.get_team_mut(&game.away.team)
                        .unwrap()
                        .shuffle_players(rng);

                    game.report("The teams are caught in the reverb!!".to_string(), data);
                }

                WeatherResult::Nothing
            }
            Self::Observation => {
                if rng.gen::<f64>() < 0.05 {
                    let old_player = data
                        .get_team(&hitter_state.team)
                        .and_then(Team::get_current_player)
                        .unwrap();

                    let new_player = data.new_player(rng);
                    *data
                        .get_team_mut(&hitter_state.team)
                        .and_then(|x| x.get_current_player_mut())
                        .unwrap() = new_player;

                    game.report(
                        format!(
                            "The observers have defragged {}.",
                            data.get_player(&old_player).unwrap().get_name(),
                        ),
                        data,
                    );

                    game.report(
                        format!(
                            "{} has been created in their place! They don't know what's going on!",
                            data.get_player(&new_player).unwrap().get_name(),
                        ),
                        data,
                    );

                    WeatherResult::Prevent
                } else if rng.gen::<f64>() < 0.1 {
                    let hitter = data
                        .get_team(&hitter_state.team)
                        .and_then(Team::get_current_player)
                        .unwrap();

                    game.report(
                        format!(
                            "The overseers watch {} with intent.",
                            data.get_player(&hitter).unwrap().get_name(),
                        ),
                        data,
                    );
                    WeatherResult::Nothing
                } else {
                    WeatherResult::Nothing
                }
            }
            Self::Omni => rng.gen::<Self>().pre_hit(hitter, game, data, rng),
            Self::Unpredictable => {
                if rng.gen::<f64>() < 0.05 {
                    game.weather = rng.gen();
                    game.weather.weather_announce(hitter, game, data);
                    game.weather.pre_hit(hitter, game, data, rng)
                } else {
                    WeatherResult::Nothing
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum WeatherResult {
    Prevent,
    Nothing,
}
