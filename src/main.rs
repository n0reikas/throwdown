mod card;
mod deck;
mod game;
mod player;

use game::Game;
use macroquad::prelude::*;
use player::Player;

#[macroquad::main("Throwdown")]
async fn main() {
    let player_list = vec![
        Player::new(0, "Alice".to_string()),
        Player::new(1, "Bob".to_string()),
        Player::new(2, "Carl".to_string()),
    ];
    let mut game = Game::new(player_list);

    game.deal();

    loop {
        clear_background(DARKGREEN);
        if let Some(kozyr_suit) = game.kozyr_suit {
            draw_text(format!("Kozyr: {}", kozyr_suit), 40.0, 40.0, 10.0, BLACK);
        }
        next_frame().await;
    }
}
