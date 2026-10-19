# Throwdown

A multiplayer card game for three players, written in Rust with [macroquad](https://macroquad.rs/).
A WebSocket server runs the game and each player connects with their own client window.

## Running

**Rust** must be installed. See [Rust install page](https://rust-lang.org/tools/install/) for
install info.

Start the server:

```sh
cargo run -p server
```

It listens on `ws://localhost:9001`. Then start one client per player, each in its own
terminal, passing the player's name after `--`:

```sh
cargo run -p client -- Alice
cargo run -p client -- Bob
cargo run -p client -- Carl
```

Each window shows "Waiting for players..." until all three have joined, then the game starts.

### Testing the server without a client

The server can also be driven by hand with [websocat](https://github.com/vi/websocat)
(`cargo install websocat`). Connect from three terminals with `websocat ws://localhost:9001`
and send JSON messages, one per line:

```json
{"Join":{"name":"Alice"}}
{"Act":{"PlayCard":0}}
{"Act":"TakeHome"}
```

## Project layout

The repository is a Cargo workspace with three crates:

- `core/` (`throwdown-core`): game rules, cards, and the client/server message types
  (`protocol.rs`). No graphics or networking.
- `server/`: accepts WebSocket connections, owns the only real `Game`, validates moves and
  sends each player a view of the game that hides the other players' hands.
- `client/`: the macroquad window. Draws the latest view from the server and sends clicks
  back as actions.

Run the rules tests with `cargo test -p throwdown-core`.

## How it plays

**Dealing.** The 52-card deck is shuffled and players take turns drawing onto a face-up pile.
If a drawn card (or the top of your pile) is exactly one rank above another player's top card
(2 counts as one above Ace), you gift it onto their pile and keep going. The suit of the last
card in the deck becomes the *kozyr* (trump). When the deck runs out, each pile becomes that
player's hand, and whoever holds the 2 of kozyr goes first.

**Battle.** On your turn, either play a card onto the center stack or click **Take Home** to
pick up the bottom card of the stack. A card can be played on an empty stack, or on top of:

- a lower card of the same suit, or
- any non-kozyr, non-club card, if it is a kozyr.

Clubs can only be beaten by higher clubs. If kozyr is clubs, there is no kozyr at all.
Once every player has a card on the stack, it is cleared and the last player leads again.

If a player successfully clears their hand, they win. The last player to stay with cards still
in their hand loses.

## Controls

Click a card in your hand to play it, or click **Take Home**. Each window shows only its own
player's hand, and clicks only work on your turn.

## Status

Work in progress:

- no win condition or game-over screen yet
- dealing happens instantly instead of being animated
- the client always connects to `localhost:9001`, so all players must be on the same machine
- no reconnecting: if a player leaves mid-game, the game can't continue
