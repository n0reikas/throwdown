# Throwdown

A hot-seat card game for three players, written in Rust with [macroquad](https://macroquad.rs/).

## Running

**Rust** must be installed. See [Rust install page](https://rust-lang.org/tools/install/) for
install info.

Run this in your terminal to play:

```sh
cargo run
```

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

Click a card in your hand to play it, or click **Take Home**. The hand of the player whose
turn it is is shown at the bottom of the screen; it slides away between turns so players
can pass the device.

## Status

Work in progress: there is no win condition or game-over screen yet, and player names are
hard-coded in `main.rs`.
