mod card;
mod deck;
mod game;
mod player;

use std::collections::HashMap;

use crate::card::Card;
use crate::game::Action;
use game::Game;
use macroquad::prelude::*;
use player::Player;

const FONT_SIZE: f32 = 20.0;
const TEXT_COLOR: Color = BLACK;
const CARD_WIDTH: f32 = 60.0;
const CARD_HEIGHT: f32 = 90.0;

const TURN_DELAY: f32 = 0.5;
const HAND_HIDE_TIME: f32 = 0.35;
const HAND_HIDE_OFFSET: f32 = 250.0;

const CARD_RAISE_OFFSET: f32 = -25.0;

const FAN_RADIUS: f32 = 400.0;
const FAN_STEP: f32 = 0.08;
const FAN_MAX_ANGLE: f32 = 0.8;

#[derive(Clone, Copy)]
struct Pose {
    center: Vec2,
    rotation: f32,
}

impl Pose {
    fn lerp(self, to: Pose, t: f32) -> Pose {
        Pose {
            center: self.center.lerp(to.center, t),
            rotation: self.rotation + (to.rotation - self.rotation) * t,
        }
    }

    fn lowered(self) -> Pose {
        Pose {
            center: self.center + vec2(0.0, HAND_HIDE_OFFSET),
            ..self
        }
    }

    fn raised(self) -> Pose {
        Pose {
            center: self.center
                + Vec2::from_angle(self.rotation).rotate(vec2(0.0, CARD_RAISE_OFFSET)),
            ..self
        }
    }

    fn contains(&self, point: Vec2) -> bool {
        let local = Vec2::from_angle(-self.rotation).rotate(point - self.center);
        local.x.abs() <= CARD_WIDTH / 2.0 && local.y.abs() <= CARD_HEIGHT / 2.0
    }
}

#[macroquad::main("Throwdown")]
async fn main() {
    let player_list = vec![
        Player::new(0, "Alice".to_string()),
        Player::new(1, "Bob".to_string()),
        Player::new(2, "Carl".to_string()),
    ];
    let mut game = Game::new(player_list);

    game.deal();
    let mut card_pos: HashMap<Card, Pose> = HashMap::new();
    let mut shown_player = game.current_player_index;
    let mut switch_timer = 0.0;

    loop {
        let dt = get_frame_time();
        let t = 1.0 - (-12.0 * dt).exp();

        if shown_player != game.current_player_index {
            switch_timer += dt;
            if switch_timer >= TURN_DELAY + HAND_HIDE_TIME {
                switch_timer = 0.0;
                shown_player = game.current_player_index;
                let hand = &game.players[shown_player].hand;
                for (card, pose) in hand.iter().zip(hand_layout(hand.len())) {
                    card_pos.insert(*card, pose.lowered());
                }
            }
        }
        let hand_hidden = shown_player != game.current_player_index && switch_timer >= TURN_DELAY;

        let hand_len = game.players[shown_player].hand.len();
        let stack_len = game.center_stack.len();
        let discard_len = game.discard_pile.len();
        let hand_poses = hand_layout(hand_len);
        let stack_poses = stack_layout(stack_len);
        let discard_poses = discard_layout(discard_len);
        let take_home_rect = Rect::new(600.0, 400.0, 100.0, 40.0);

        let mouse = Vec2::from(mouse_position());
        let hovered = if shown_player == game.current_player_index {
            hand_poses.iter().rposition(|pose| pose.contains(mouse))
        } else {
            None
        };
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = hovered {
                let _ = game.apply(Action::PlayCard(i));
            } else if take_home_rect.contains(mouse) && shown_player == game.current_player_index {
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
        let current_player = &game.players[shown_player];
        draw_text(
            format!("{}'s turn", current_player.name),
            40.0,
            40.0 + FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
        );

        for (i, (card, target)) in current_player
            .hand
            .iter()
            .zip(hand_poses.iter())
            .enumerate()
        {
            let target = if hand_hidden {
                target.lowered()
            } else {
                *target
            };
            let target = if hovered == Some(i) {
                target.raised()
            } else {
                target
            };
            let pose = card_pos.entry(*card).or_insert(target);
            *pose = pose.lerp(target, t);
            draw_card(card, *pose);
        }

        for (card, target) in game.center_stack.iter().zip(stack_poses.iter()) {
            let pose = card_pos.entry(*card).or_insert(*target);
            *pose = pose.lerp(*target, t);
            draw_card(card, *pose);
        }

        for (card, target) in game.discard_pile.iter().zip(discard_poses.iter()) {
            let pose = card_pos.entry(*card).or_insert(*target);
            *pose = pose.lerp(*target, t);
            draw_card_back(*pose);
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

fn draw_card(card: &Card, pose: Pose) {
    let params = DrawRectangleParams {
        offset: vec2(0.5, 0.5),
        rotation: pose.rotation,
        color: WHITE,
    };
    draw_rectangle_ex(
        pose.center.x,
        pose.center.y,
        CARD_WIDTH,
        CARD_HEIGHT,
        params.clone(),
    );
    draw_rectangle_lines_ex(
        pose.center.x,
        pose.center.y,
        CARD_WIDTH,
        CARD_HEIGHT,
        2.0,
        DrawRectangleParams {
            color: BLACK,
            ..params
        },
    );

    // label sits in the top-left corner, which stays visible when cards overlap
    let corner = vec2(-CARD_WIDTH / 2.0 + 4.0, -CARD_HEIGHT / 2.0 + FONT_SIZE);
    let label = pose.center + Vec2::from_angle(pose.rotation).rotate(corner);
    draw_text_ex(
        card.short_label(),
        label.x,
        label.y,
        TextParams {
            font_size: FONT_SIZE as u16,
            color: TEXT_COLOR,
            rotation: pose.rotation,
            ..Default::default()
        },
    );
}

fn draw_card_back(pose: Pose) {
    let params = DrawRectangleParams {
        offset: vec2(0.5, 0.5),
        rotation: pose.rotation,
        color: MAROON,
    };
    draw_rectangle_ex(
        pose.center.x,
        pose.center.y,
        CARD_WIDTH,
        CARD_HEIGHT,
        params.clone(),
    );
    draw_rectangle_lines_ex(
        pose.center.x,
        pose.center.y,
        CARD_WIDTH,
        CARD_HEIGHT,
        2.0,
        DrawRectangleParams {
            color: BLACK,
            ..params
        },
    );
}

// cards sit on an arc around a pivot below the screen, each one tilted outwards
fn hand_layout(card_count: usize) -> Vec<Pose> {
    let mut cards = Vec::new();
    let step = if card_count > 1 {
        FAN_STEP.min(FAN_MAX_ANGLE / (card_count - 1) as f32)
    } else {
        0.0
    };
    let pivot = vec2(screen_width() / 2.0, screen_height() - 90.0 + FAN_RADIUS);
    for i in 0..card_count {
        let angle = (i as f32 - (card_count - 1) as f32 / 2.0) * step;
        let center = pivot + vec2(angle.sin(), -angle.cos()) * FAN_RADIUS;
        cards.push(Pose {
            center,
            rotation: angle,
        });
    }
    cards
}

fn stack_layout(card_count: usize) -> Vec<Pose> {
    let mut cards = Vec::new();
    let mut y = 100.0;
    let width = CARD_WIDTH;
    let height = CARD_HEIGHT;
    for i in 0..card_count {
        let x = i as f32 * 10.0 + width;
        y += i as f32 * 10.0;
        cards.push(Pose {
            center: vec2(x + width / 2.0, y + height / 2.0),
            rotation: 0.0,
        });
    }
    cards
}

fn discard_layout(card_count: usize) -> Vec<Pose> {
    let mut cards = Vec::new();
    let x = 750.0;
    let y = 300.0;
    let width = CARD_WIDTH;
    let height = CARD_HEIGHT;
    for i in 0..card_count {
        cards.push(Pose {
            center: vec2(x + width / 2.0, y + height / 2.0),
            rotation: ((i as f32) * 1.7).sin() * 0.15,
        });
    }
    cards
}
