use crate::card::{Card, Suit};
use crate::game::{Action, GameState};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMsg {
    Join { name: String },
    Act(Action),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { seat: usize },
    State(PlayerView),
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct PlayerView {
    pub my_seat: usize,
    pub my_hand: Vec<Card>,
    pub players: Vec<SeatView>,
    pub center_stack: Vec<Card>,
    pub discard_count: usize,
    pub kozyr_suit: Option<Suit>,
    pub current_player: usize,
    pub state: GameState,
}

#[derive(Serialize, Deserialize)]
pub struct SeatView {
    pub name: String,
    pub hand_count: usize,
    pub face_up_stack: Vec<Card>,
}
