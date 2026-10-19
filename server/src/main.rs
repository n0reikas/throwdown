use std::collections::HashMap;

use futures_util::{SinkExt, StreamExt};
use throwdown_core::{
    game::Game,
    player::Player,
    protocol::{ClientMsg, ServerMsg},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::tungstenite::Message;

const ADDR: &str = "0.0.0.0:9001";
const NUM_PLAYERS: usize = 3;

type ClientTx = mpsc::UnboundedSender<ServerMsg>;

enum Event {
    Connected { id: usize, tx: ClientTx },
    Msg { id: usize, msg: ClientMsg },
    Disconnected { id: usize },
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR).await.expect("failed to bind");
    println!("listening on ws://{ADDR}");

    let (event_tx, event_rx) = mpsc::unbounded_channel();
    tokio::spawn(game_loop(event_rx));

    let mut next_id = 0;
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, next_id, event_tx.clone()));
        next_id += 1;
    }
}

async fn handle_connection(stream: TcpStream, id: usize, events: mpsc::UnboundedSender<Event>) {
    let Ok(ws) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();
    let _ = events.send(Event::Connected { id, tx });

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).expect("ServerMsg serializes");
            if ws_tx.send(Message::text(json)).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMsg>(&text) {
                Ok(msg) => {
                    let _ = events.send(Event::Msg { id, msg });
                }
                Err(e) => eprintln!("bad message from {id}: {e}"),
            }
        }
    }

    let _ = events.send(Event::Disconnected { id });
    writer.abort();
}

async fn game_loop(mut events: mpsc::UnboundedReceiver<Event>) {
    let mut clients: HashMap<usize, ClientTx> = HashMap::new();
    let mut seats: Vec<(usize, String)> = Vec::new();
    let mut game: Option<Game> = None;

    while let Some(event) = events.recv().await {
        match event {
            Event::Connected { id, tx } => {
                clients.insert(id, tx);
            }

            Event::Disconnected { id } => {
                clients.remove(&id);
                let Some(seat) = seats.iter().position(|(c, _)| *c == id) else {
                    continue;
                };
                if game.is_none() {
                    seats.remove(seat);
                    for (seat, (c, _)) in seats.iter().enumerate() {
                        send(&clients, *c, ServerMsg::Welcome { seat });
                    }
                } else {
                    println!("{} left mid-game", seats[seat].1);
                }
            }

            Event::Msg {
                id,
                msg: ClientMsg::Join { name },
            } => {
                if game.is_some() {
                    send(
                        &clients,
                        id,
                        ServerMsg::Error("game already started".into()),
                    );
                    continue;
                }
                if seats.iter().any(|(c, _)| *c == id) {
                    send(&clients, id, ServerMsg::Error("already joined".into()));
                    continue;
                }

                let seat = seats.len();
                println!("{name} joined as seat {seat}");
                seats.push((id, name));
                send(&clients, id, ServerMsg::Welcome { seat });

                if seats.len() == NUM_PLAYERS {
                    let players = seats
                        .iter()
                        .enumerate()
                        .map(|(seat, (_, name))| Player::new(seat, name.clone()))
                        .collect();
                    let mut new_game = Game::new(players);
                    new_game.deal();
                    broadcast(&clients, &seats, &new_game);
                    game = Some(new_game);
                }
            }

            Event::Msg {
                id,
                msg: ClientMsg::Act(action),
            } => {
                let Some(game) = game.as_mut() else {
                    send(&clients, id, ServerMsg::Error("game hasn't started".into()));
                    continue;
                };
                let Some(seat) = seats.iter().position(|(c, _)| *c == id) else {
                    send(&clients, id, ServerMsg::Error("you're not seated".into()));
                    continue;
                };
                if seat != game.current_player_index {
                    send(&clients, id, ServerMsg::Error("not your turn".into()));
                    continue;
                }

                match game.apply(action) {
                    Ok(()) => broadcast(&clients, &seats, game),
                    Err(_) => send(&clients, id, ServerMsg::Error("illegal move".into())),
                }
            }
        }
    }
}

fn send(clients: &HashMap<usize, ClientTx>, id: usize, msg: ServerMsg) {
    if let Some(tx) = clients.get(&id) {
        let _ = tx.send(msg);
    }
}

fn broadcast(clients: &HashMap<usize, ClientTx>, seats: &[(usize, String)], game: &Game) {
    for (seat, (id, _)) in seats.iter().enumerate() {
        send(clients, *id, ServerMsg::State(game.view_for(seat)));
    }
}
