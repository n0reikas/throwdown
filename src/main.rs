mod card;
mod deck;
mod game;
mod player;

use std::mem::take;

use crate::game::Action;
use game::Game;
use macroquad::prelude::*;
use player::Player;

const FONT_SIZE: f32 = 20.0;
const TEXT_COLOR: Color = BLACK;
const CARD_WIDTH: f32 = 60.0;
const CARD_HEIGHT: f32 = 90.0;

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
        let hand_len = game.players[game.current_player_index].hand.len();
        let stack_len = game.center_stack.len();
        let hand_rects = hand_layout(hand_len);
        let stack_rects = stack_layout(stack_len);
        let take_home_rect = Rect::new(600.0, 400.0, 100.0, 40.0);

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            for (i, rect) in hand_rects.iter().enumerate() {
                if rect.contains(vec2(mouse_x, mouse_y)) {
                    let _ = game.apply(Action::PlayCard(i));
                    break;
                }
            }
            if take_home_rect.contains(vec2(mouse_x, mouse_y)) {
                let _ = game.apply(Action::TakeHome);
            }
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

        for (card, rect) in current_player.hand.iter().zip(hand_rects.iter()) {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, WHITE);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BLACK);
            draw_text(
                card.short_label(),
                rect.x,
                rect.y + 45.0,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }

        for (card, rect) in game.center_stack.iter().zip(stack_rects.iter()) {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, WHITE);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BLACK);
            draw_text(
                card.short_label(),
                rect.x,
                rect.y + 45.0,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }

        if !game.center_stack.is_empty() {
            draw_rectangle(
                take_home_rect.x,
                take_home_rect.y,
                take_home_rect.w,
                take_home_rect.h,
                WHITE,
            );
            draw_rectangle_lines(
                take_home_rect.x,
                take_home_rect.y,
                take_home_rect.w,
                take_home_rect.h,
                2.0,
                BLACK,
            );
            draw_text(
                "Take Home",
                take_home_rect.x + 5.0,
                take_home_rect.y + 20.0,
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
    let width = CARD_WIDTH;
    let height = CARD_HEIGHT;
    for i in 0..card_count {
        let x = i as f32 * width;
        let r = Rect::new(x, y, width, height);
        cards.push(r);
    }
    cards
}

fn stack_layout(card_count: usize) -> Vec<Rect> {
    let mut cards = Vec::new();
    let mut y = 100.0;
    let width = CARD_WIDTH;
    let height = CARD_HEIGHT;
    for i in 0..card_count {
        let x = i as f32 * 10.0 + width;
        y += i as f32 * 10.0;
        let r = Rect::new(x, y, width, height);
        cards.push(r);
    }
    cards
}
