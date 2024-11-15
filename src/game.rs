mod report;

use std::collections::VecDeque;

use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};
use report::Report;

use crate::{
    team::{Team, TeamId},
    Data,
};

// I plan to support multiple kinds of games so i'm making this a struct
pub trait RunGame {
    fn tick(&mut self, data: &mut Data);

    fn report(&mut self, comment: String, data: &Data);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Space {
    First,
    Middle,
    Third,
}

impl Space {
    pub fn farthest(self) -> Self {
        match self {
            Self::First => Self::Third,
            Self::Middle => loop {
                let x: Self = thread_rng().gen();
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
    Playing(Side),
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
}

impl Distribution<Weather> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weather {
        match rng.gen_range::<usize, _>(0..=3) {
            0 => Weather::None,
            1 => Weather::Feedback,
            2 => Weather::Reverb,
            3 => Weather::Observation,
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

#[allow(clippy::too_many_lines)]
// ignore the fact that this is a trait and not a impl on the Game itself. i want to support having multiple kinds of games eventually
impl RunGame for Game {
    fn tick(&mut self, data: &mut Data) {
        match self.state {
            GameState::Serving(serving_side) => {
                let (serving_state, receiving_state) = match serving_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };
                serving_state.space = Space::Middle;
                receiving_state.space = Space::Middle;
                self.ball_direction = thread_rng().gen();

                let serving_player_name = serving_state
                    .team
                    .get_current_player(data)
                    .and_then(|x| data.get_player(&x))
                    .map(|x| x.get_name())
                    .unwrap();

                self.report(format!("{} serves!", serving_player_name), data);

                self.state = GameState::Playing(serving_side.opposite());
            }
            GameState::Playing(hitting_side) => {
                let (hitter_state, _) = match hitting_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                let hitting_player = hitter_state.team.get_current_player(data).unwrap();

                if hitter_state.space == self.ball_direction {
                    if data
                        .get_player(&hitting_player)
                        .unwrap()
                        .distraction_check()
                    {
                        hitter_state.space = thread_rng().gen();
                    }
                } else if data.get_player(&hitting_player).unwrap().speed_check() {
                    hitter_state.space = self.ball_direction;
                } else {
                    hitter_state.space = thread_rng().gen();
                }

                let weather_result = self.weather.pre_hit(hitting_side, self, data);

                // This might change after weather events

                let (hitter_state, _) = match hitting_side {
                    Side::Home => (&mut self.home, &mut self.away),
                    Side::Away => (&mut self.away, &mut self.home),
                };

                let hitting_player = hitter_state.team.get_current_player(data).unwrap();

                match weather_result {
                    WeatherResult::Prevent => {
                        self.report(
                            format!(
                                "{} doesn't manage to hit.",
                                data.get_player(&hitting_player).unwrap().get_name()
                            ),
                            data,
                        );
                        self.get_team_mut(hitting_side.opposite()).score += 1;

                        self.report(
                            format!(
                                "{} scores!",
                                data.get_player(
                                    &self
                                        .get_team(hitting_side.opposite())
                                        .team
                                        .get_current_player(data)
                                        .unwrap()
                                )
                                .unwrap()
                                .get_name()
                            ),
                            data,
                        );
                        self.state = GameState::Serving(hitting_side.opposite());
                    }
                    WeatherResult::Nothing => {
                        if hitter_state.space == self.ball_direction {
                            if data.get_player(&hitting_player).unwrap().control_check() {
                                self.ball_direction = self.ball_direction.farthest();
                            } else {
                                self.ball_direction = thread_rng().gen();
                            }
                            self.report(
                                format!(
                                    "{} hits!",
                                    data.get_player(&hitting_player).unwrap().get_name()
                                ),
                                data,
                            );
                            self.state = GameState::Playing(hitting_side.opposite());
                        } else {
                            self.report(
                                format!(
                                    "{} fails to hit it!",
                                    data.get_player(&hitting_player).unwrap().get_name()
                                ),
                                data,
                            );
                            self.get_team_mut(hitting_side.opposite()).score += 1;
                            self.report(
                                format!(
                                    "{} scores!",
                                    data.get_player(&hitting_player).unwrap().get_name()
                                ),
                                data,
                            );
                            self.state = GameState::Serving(hitting_side.opposite());
                        }
                    }
                }
            }
        }
    }

    fn report(&mut self, comment: String, data: &Data) {
        let report = Report::take_snapshot(self, data).with_comment(comment);
        self.reports.push_front(report);
    }
}

impl Weather {
    fn pre_hit(self, hitter: Side, game: &mut Game, data: &mut Data) -> WeatherResult {
        let (hitter_state, _) = match hitter {
            Side::Home => (&mut game.home, &mut game.away),
            Side::Away => (&mut game.away, &mut game.home),
        };
        match self {
            Self::None => WeatherResult::Nothing,
            Self::Feedback => {
                if thread_rng().gen::<f64>() < 0.05 {
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
                if thread_rng().gen::<f64>() < 0.05 {
                    data.get_team_mut(&game.home.team)
                        .unwrap()
                        .shuffle_players();
                    data.get_team_mut(&game.away.team)
                        .unwrap()
                        .shuffle_players();

                    game.report("The teams are caught in the reverb!!".to_string(), data);
                }

                WeatherResult::Nothing
            }
            Self::Observation => {
                if thread_rng().gen::<f64>() < 0.05 {
                    let old_player = data
                        .get_team(&hitter_state.team)
                        .and_then(Team::get_current_player)
                        .unwrap();

                    let new_player = data.new_player();
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
                } else if thread_rng().gen::<f64>() < 0.1 {
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
            Self::Omni => thread_rng().gen::<Self>().pre_hit(hitter, game, data),
        }
    }
}

pub enum WeatherResult {
    Prevent,
    Nothing,
}
