use std::collections::HashMap;

use ewebsock::{WsEvent, WsMessage, connect};
use macroquad::prelude::*;
use throwdown_core::card::Card;
use throwdown_core::game::{Action, Game};
use throwdown_core::player::Player;
use throwdown_core::protocol::{ClientMsg, PlayerView, ServerMsg};

const VIRTUAL_HEIGHT: f32 = 600.0;

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

fn window_conf() -> Conf {
    Conf {
        window_height: 1080,
        window_width: 1920,
        window_title: "Throwdown".to_string(),
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

fn send(ws_tx: &mut ewebsock::WsSender, msg: &ClientMsg) {
    ws_tx.send(WsMessage::Text(serde_json::to_string(msg).unwrap()));
}

#[macroquad::main(window_conf)]
async fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Player".to_string());
    let (mut ws_tx, ws_rx) =
        connect("ws://localhost:9001", ewebsock::Options::default()).expect("failed to connect");

    let mut view: Option<PlayerView> = None;
    let mut card_pos: HashMap<Card, Pose> = HashMap::new();

    loop {
        while let Some(event) = ws_rx.try_recv() {
            match event {
                WsEvent::Opened => {
                    let join = ClientMsg::Join { name: name.clone() };
                    ws_tx.send(WsMessage::Text(serde_json::to_string(&join).unwrap()));
                }
                WsEvent::Message(WsMessage::Text(text)) => {
                    match serde_json::from_str::<ServerMsg>(&text) {
                        Ok(ServerMsg::Welcome { seat }) => println!("seated at {seat}"),
                        Ok(ServerMsg::State(v)) => view = Some(v),
                        Ok(ServerMsg::Error(e)) => println!("server: {e}"),
                        Err(e) => eprintln!("bad message: {e}"),
                    }
                }
                WsEvent::Error(e) => eprintln!("connection error: {e}"),
                WsEvent::Closed => println!("disconnected"),
                _ => {}
            }
        }

        let dt = get_frame_time();
        let t = 1.0 - (-12.0 * dt).exp();

        let virtual_w = VIRTUAL_HEIGHT * screen_width() / screen_height();

        let camera = Camera2D {
            target: vec2(virtual_w / 2.0, VIRTUAL_HEIGHT / 2.0),
            zoom: vec2(2.0 / virtual_w, 2.0 / VIRTUAL_HEIGHT),
            ..Default::default()
        };

        set_camera(&camera);
        clear_background(DARKGREEN);

        let Some(view) = view.as_ref() else {
            draw_text("Waiting for players...", 40.0, 40.0, FONT_SIZE, TEXT_COLOR);
            next_frame().await;
            continue;
        };

        let my_turn = view.current_player == view.my_seat;
        let hand_poses = hand_layout(view.my_hand.len(), virtual_w);
        let stack_poses = stack_layout(view.center_stack.len());
        let discard_poses = discard_layout(view.discard_count, virtual_w);
        let take_home_rect = Rect::new(600.0, 400.0, 100.0, 40.0);

        let mouse = camera.screen_to_world(Vec2::from(mouse_position()));
        let hovered = if my_turn {
            hand_poses.iter().rposition(|pose| pose.contains(mouse))
        } else {
            None
        };
        if my_turn && is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = hovered {
                send(&mut ws_tx, &ClientMsg::Act(Action::PlayCard(i)));
            } else if take_home_rect.contains(mouse) && !view.center_stack.is_empty() {
                send(&mut ws_tx, &ClientMsg::Act(Action::TakeHome));
            }
        }

        if let Some(kozyr_suit) = view.kozyr_suit {
            draw_text(
                format!("Kozyr: {}", kozyr_suit),
                40.0,
                40.0,
                FONT_SIZE,
                TEXT_COLOR,
            );
        }
        let turn_text = if my_turn {
            "Your turn".to_string()
        } else {
            format!("{}'s turn", view.players[view.current_player].name)
        };
        draw_text(turn_text, 40.0, 40.0 + FONT_SIZE, FONT_SIZE, TEXT_COLOR);

        for (i, (card, target)) in view.my_hand.iter().zip(hand_poses.iter()).enumerate() {
            let target = if hovered == Some(i) {
                target.raised()
            } else {
                *target
            };
            let pose = card_pos.entry(*card).or_insert(target);
            *pose = pose.lerp(target, t);
            draw_card(card, *pose);
        }

        for (card, target) in view.center_stack.iter().zip(stack_poses.iter()) {
            let pose = card_pos.entry(*card).or_insert(*target);
            *pose = pose.lerp(*target, t);
            draw_card(card, *pose);
        }

        for pose in &discard_poses {
            draw_card_back(*pose);
        }

        if my_turn && !view.center_stack.is_empty() {
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
fn hand_layout(card_count: usize, virtual_w: f32) -> Vec<Pose> {
    let mut cards = Vec::new();
    let step = if card_count > 1 {
        FAN_STEP.min(FAN_MAX_ANGLE / (card_count - 1) as f32)
    } else {
        0.0
    };
    let pivot = vec2(virtual_w / 2.0, VIRTUAL_HEIGHT - 90.0 + FAN_RADIUS);
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

fn discard_layout(card_count: usize, virtual_w: f32) -> Vec<Pose> {
    let mut cards = Vec::new();
    let x = virtual_w - CARD_WIDTH / 2.0;
    let y = VIRTUAL_HEIGHT / 2.0 - CARD_HEIGHT;
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
