mod report;
mod states;
mod weather;

use std::collections::VecDeque;

use rand::Rng;
use report::Report;
use states::{PlayerState, Space};
use weather::{Weather, WeatherResult};

use crate::{player::Player, team::TeamId, Data};

// I plan to support multiple kinds of games so i'm making this a struct
pub trait Run {
    fn tick<R: Rng>(&mut self, data: &mut Data, rng: &mut R) -> Result;

    fn report(&mut self, comment: String, data: &Data);
}

enum GameState {
    Serving(Side),
    PreHit(Side),
    Hit(Side, WeatherResult),
    Score(Side),
}

#[derive(Clone, Copy)]
pub enum Side {
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
    pub const fn get_team(&self, team: Side) -> &PlayerState {
        match team {
            Side::Home => &self.home,
            Side::Away => &self.away,
        }
    }
    pub fn get_team_mut(&mut self, team: Side) -> &mut PlayerState {
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
