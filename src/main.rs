mod card;
mod deck;
mod game;
mod player;

use game::Game;
use macroquad::prelude::*;
use player::Player;

const FONT_SIZE: f32 = 20.0;
const TEXT_COLOR: Color = BLACK;

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
            draw_text(
                format!("Kozyr: {}", kozyr_suit),
                40.0,
                40.0,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }
        let current_player = &game.players[game.current_player_index];
        draw_text(
            format!("{}'s turn", current_player.name),
            40.0,
            40.0 + FONT_SIZE,
            FONT_SIZE,
            BLACK,
        );

        for (i, card) in current_player.hand.iter().enumerate() {
            draw_text(
                format!("{card}"),
                50.0,
                40.0 + FONT_SIZE * 2.0 + i as f32 * FONT_SIZE,
                FONT_SIZE,
                BLACK,
            );
        }
        next_frame().await;
    }
}
