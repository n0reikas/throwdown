mod card;
mod deck;
mod game;
mod player;

use crate::game::Action;
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
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            // let _ = game.apply(Action::PlayCard(i));
        }
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
            TEXT_COLOR,
        );

        let rects = hand_layout(current_player.hand.len());

        for (card, rect) in current_player.hand.iter().zip(rects.iter()) {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, WHITE);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BLACK);
            draw_text(
                format!("{card}"),
                rect.x,
                rect.y + 45.0,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }

        for (i, card) in game.center_stack.iter().enumerate() {
            draw_text(
                format!("{card}"),
                80.0,
                40.0 + FONT_SIZE * 2.0 + i as f32 * FONT_SIZE,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }
        next_frame().await;
    }
}

fn hand_layout(card_count: usize) -> Vec<Rect> {
    let mut cards = Vec::new();
    let y = 500.0;
    let width = 60.0;
    let height = 90.0;
    for i in 0..card_count {
        let x = i as f32 * width;
        let r = Rect::new(x, y, width, height);
        cards.push(r);
    }
    cards
}
