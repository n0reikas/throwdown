use crate::card::Card;
use crate::card::Suit;
use crate::deck::Deck;
use crate::player::Player;

use macroquad::prelude::*;
use std::collections::VecDeque;
use std::io;

pub enum GameState {
    Dealing,
    Playing,
    GameOver,
}

pub enum Action {
    PlayCard(usize),
    TakeHome,
}

pub enum MoveError {
    IllegalCard,
    NothingToTakeHome,
}

pub struct Game {
    pub state: GameState,
    pub deck: Deck,
    pub players: Vec<Player>,
    pub kozyr_suit: Option<Suit>,
    pub center_stack: VecDeque<Card>,
    pub current_player_index: usize,
}

impl Game {
    pub fn new(players: Vec<Player>) -> Self {
        Game {
            state: GameState::Dealing,
            deck: Deck::new(),
            players: players,
            kozyr_suit: None,
            center_stack: VecDeque::new(),
            current_player_index: 0,
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<(), MoveError> {
        match action {
            Action::PlayCard(i) => {
                if !self.can_play(i) {
                    Err(MoveError::IllegalCard)
                } else {
                    self.play_card(i);
                    Ok(())
                }
            }
            Action::TakeHome => {
                if self.center_stack.is_empty() {
                    Err(MoveError::NothingToTakeHome)
                } else {
                    self.take_home();
                    Ok(())
                }
            }
        }
    }

    fn play_card(&mut self, i: usize) {
        let chosen_card = self.players[self.current_player_index].hand.remove(i);
        self.center_stack.push_back(chosen_card);
        if self.center_stack.len() == self.players.len() {
            self.center_stack.clear();
        } else {
            self.current_player_index = self.next_player();
        }
    }

    fn take_home(&mut self) {
        self.players[self.current_player_index].hand.push(
            self.center_stack
                .pop_front()
                .expect("Center stack expected non-empty"),
        );
        self.current_player_index = self.next_player();
    }

    pub fn can_play(&self, hand_index: usize) -> bool {
        let Some(card) = self.players[self.current_player_index].hand.get(hand_index) else {
            return false;
        };

        match (self.center_stack.back(), self.kozyr_suit) {
            (None, _) => true,
            (Some(top), Some(kozyr)) => Game::is_valid_play(card, top, kozyr),
            (Some(_), None) => false,
        }
    }

    pub fn deal(&mut self) {
        self.deck.shuffle();
        while let GameState::Dealing = self.state {
            self.deal_turn();
        }
    }

    fn deal_turn(&mut self) {
        let current_index = self.current_player_index;

        loop {
            let my_top_rank = match self.players[current_index].top_card() {
                Some(card) => card.rank,
                None => break,
            };

            if let Some(target_index) = self.find_gift_target(current_index, my_top_rank) {
                let gifted_card = self.players[current_index].face_up_stack.pop().unwrap();
                println!(
                    "{} gifts {} to {}",
                    self.players[current_index].name.trim().to_string(),
                    gifted_card,
                    self.players[target_index].name.trim().to_string(),
                );
                self.players[target_index].face_up_stack.push(gifted_card);
            } else {
                break;
            }
        }

        loop {
            let drawn_card = match self.deck.draw() {
                Some(card) => card,
                None => {
                    for player in &mut self.players {
                        player.hand.append(&mut player.face_up_stack);
                    }
                    self.state = GameState::Playing;
                    println!("Battle starting.");
                    self.current_player_index = self
                        .find_kozyr_two_holder()
                        .expect("Unknown trump suit or the deck is not empty");
                    if let Some(suit) = self.kozyr_suit {
                        println!("{} is the Trump Suit", suit);
                        println!(
                            "{} starts, because they have the 2 of {}",
                            self.players[self.current_player_index]
                                .name
                                .trim()
                                .to_string(),
                            suit
                        );
                    }
                    return;
                }
            };

            if self.deck.cards.is_empty() {
                self.kozyr_suit = Some(drawn_card.suit);
            }

            println!(
                "{} draws {}",
                self.players[current_index].name.trim().to_string(),
                drawn_card,
            );

            if let Some(target_index) = self.find_gift_target(current_index, drawn_card.rank) {
                println!(
                    "{} gifts {} to {}",
                    self.players[current_index].name.trim().to_string(),
                    drawn_card,
                    self.players[target_index].name.trim().to_string(),
                );
                self.players[target_index].face_up_stack.push(drawn_card);
                continue;
            } else {
                self.players[self.current_player_index]
                    .face_up_stack
                    .push(drawn_card);
                self.current_player_index = self.next_player();
                return;
            }
        }
    }

    fn is_plus_one(rank_a: u8, rank_b: u8) -> bool {
        if rank_b == 14 {
            rank_a == 2
        } else {
            rank_a == rank_b + 1
        }
    }

    fn find_gift_target(&self, current_player_index: usize, card_to_give: u8) -> Option<usize> {
        let num_players = self.players.len();

        for offset in 1..num_players {
            let target_index = (current_player_index + offset) % num_players;

            if let Some(target_card) = self.players[target_index].top_card() {
                if Game::is_plus_one(card_to_give, target_card.rank) {
                    return Some(target_index);
                }
            }
        }
        None
    }

    fn next_player(&self) -> usize {
        (self.current_player_index + 1) % self.players.len()
    }

    fn find_kozyr_two_holder(&self) -> Option<usize> {
        if let Some(kozyr_suit) = self.kozyr_suit {
            let card = Card {
                suit: kozyr_suit,
                rank: 2,
            };
            for player in &self.players {
                if player.hand.contains(&card) {
                    return Some(player.id);
                }
            }
            None
        } else {
            None
        }
    }

    fn is_valid_play(played: &Card, top: &Card, kozyr_suit: Suit) -> bool {
        if kozyr_suit != Suit::Clubs {
            if played.suit == kozyr_suit && top.suit != kozyr_suit && top.suit != Suit::Clubs {
                return true;
            }
        }
        if played.suit == top.suit {
            return played.rank > top.rank;
        }
        false
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    fn card(rank: u8, suit: Suit) -> Card {
        Card { rank, suit }
    }

    fn test_game(hands: Vec<Vec<Card>>, kozyr: Suit) -> Game {
        let mut players = Vec::new();
        for (i, hand) in hands.into_iter().enumerate() {
            let mut player = Player::new(i, format!("Player {i}"));
            player.hand = hand;
            players.push(player);
        }

        let mut game = Game::new(players);
        game.state = GameState::Playing;
        game.kozyr_suit = Some(kozyr);
        game
    }

    // apply: PlayCard

    #[test]
    fn play_card_moves_it_from_hand_to_stack() {
        let mut game = test_game(
            vec![
                vec![card(5, Suit::Hearts), card(9, Suit::Diamonds)],
                vec![card(7, Suit::Hearts)],
            ],
            Suit::Spades,
        );

        let result = game.apply(Action::PlayCard(0));

        assert!(result.is_ok());
        assert_eq!(game.players[0].hand, vec![card(9, Suit::Diamonds)]);
        assert_eq!(game.center_stack, vec![card(5, Suit::Hearts)]);
    }

    #[test]
    fn play_card_passes_turn_to_next_player() {
        let mut game = test_game(
            vec![vec![card(5, Suit::Hearts)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );

        assert!(game.apply(Action::PlayCard(0)).is_ok());

        assert_eq!(game.current_player_index, 1);
    }

    #[test]
    fn last_player_passes_turn_back_to_first() {
        let mut game = test_game(
            vec![
                vec![card(3, Suit::Diamonds)],
                vec![card(4, Suit::Diamonds)],
                vec![card(10, Suit::Hearts)],
            ],
            Suit::Spades,
        );
        game.current_player_index = 2;
        game.center_stack = VecDeque::from([card(5, Suit::Hearts)]);

        assert!(game.apply(Action::PlayCard(0)).is_ok());

        assert_eq!(game.current_player_index, 0);
    }

    #[test]
    fn kozyr_can_be_played_on_higher_card() {
        let mut game = test_game(
            vec![vec![card(2, Suit::Spades)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );
        game.center_stack = VecDeque::from([card(14, Suit::Hearts)]);

        let result = game.apply(Action::PlayCard(0));

        assert!(result.is_ok());
    }

    #[test]
    fn illegal_card_is_rejected_and_nothing_changes() {
        let mut game = test_game(
            vec![vec![card(5, Suit::Hearts)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );
        game.center_stack = VecDeque::from([card(10, Suit::Hearts)]);

        let result = game.apply(Action::PlayCard(0));

        assert!(matches!(result, Err(MoveError::IllegalCard)));
        assert_eq!(game.players[0].hand, vec![card(5, Suit::Hearts)]);
        assert_eq!(game.center_stack, vec![card(10, Suit::Hearts)]);
        assert_eq!(game.current_player_index, 0);
    }

    #[test]
    fn card_index_out_of_range_is_rejected() {
        let mut game = test_game(
            vec![vec![card(5, Suit::Hearts)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );

        let result = game.apply(Action::PlayCard(5));

        assert!(matches!(result, Err(MoveError::IllegalCard)));
        assert_eq!(game.players[0].hand, vec![card(5, Suit::Hearts)]);
    }

    #[test]
    fn filling_the_stack_clears_it_and_same_player_goes_again() {
        let mut game = test_game(
            vec![
                vec![card(10, Suit::Hearts), card(3, Suit::Diamonds)],
                vec![card(7, Suit::Hearts)],
            ],
            Suit::Spades,
        );
        game.center_stack = VecDeque::from([card(5, Suit::Hearts)]);

        let result = game.apply(Action::PlayCard(0));

        assert!(result.is_ok());
        assert!(game.center_stack.is_empty());
        assert_eq!(game.current_player_index, 0);
    }

    // apply: TakeHome

    #[test]
    fn take_home_on_empty_stack_is_rejected() {
        let mut game = test_game(
            vec![vec![card(5, Suit::Hearts)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );

        let result = game.apply(Action::TakeHome);

        assert!(matches!(result, Err(MoveError::NothingToTakeHome)));
        assert_eq!(game.players[0].hand, vec![card(5, Suit::Hearts)]);
        assert_eq!(game.current_player_index, 0);
    }

    #[test]
    fn take_home_takes_the_bottom_card() {
        let mut game = test_game(
            vec![vec![card(3, Suit::Diamonds)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );
        game.center_stack = VecDeque::from([card(5, Suit::Hearts), card(10, Suit::Hearts)]);

        let result = game.apply(Action::TakeHome);

        assert!(result.is_ok());
        assert_eq!(
            game.players[0].hand,
            vec![card(3, Suit::Diamonds), card(5, Suit::Hearts)]
        );
        assert_eq!(game.center_stack, vec![card(10, Suit::Hearts)]);
    }

    #[test]
    fn take_home_passes_turn_to_next_player() {
        let mut game = test_game(
            vec![vec![card(3, Suit::Diamonds)], vec![card(7, Suit::Hearts)]],
            Suit::Spades,
        );
        game.center_stack = VecDeque::from([card(5, Suit::Hearts)]);

        assert!(game.apply(Action::TakeHome).is_ok());

        assert_eq!(game.current_player_index, 1);
    }

    #[test]
    fn same_suit_higher_rank_beats() {
        assert!(Game::is_valid_play(
            &card(10, Suit::Hearts),
            &card(9, Suit::Hearts),
            Suit::Spades,
        ));
    }

    #[test]
    fn same_suit_lower_rank_doesnt_beat() {
        assert_eq!(
            Game::is_valid_play(&card(8, Suit::Hearts), &card(9, Suit::Hearts), Suit::Spades),
            false
        );
    }

    #[test]
    fn kozyr_suit_lower_rank_beats() {
        assert!(Game::is_valid_play(
            &card(2, Suit::Spades),
            &card(9, Suit::Hearts),
            Suit::Spades,
        ));
    }

    // Rule 1: club can only beat club

    #[test]
    fn higher_club_beats_lower_club() {
        assert!(Game::is_valid_play(
            &card(10, Suit::Clubs),
            &card(5, Suit::Clubs),
            Suit::Spades,
        ));
    }

    #[test]
    fn lower_club_doesnt_beat_higher_club() {
        assert!(!Game::is_valid_play(
            &card(5, Suit::Clubs),
            &card(10, Suit::Clubs),
            Suit::Spades,
        ));
    }

    #[test]
    fn club_doesnt_beat_other_suit() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Clubs),
            &card(2, Suit::Hearts),
            Suit::Spades,
        ));
    }

    #[test]
    fn club_doesnt_beat_kozyr() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Clubs),
            &card(2, Suit::Spades),
            Suit::Spades,
        ));
    }

    #[test]
    fn other_suit_doesnt_beat_club() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Hearts),
            &card(2, Suit::Clubs),
            Suit::Spades,
        ));
    }

    #[test]
    fn kozyr_doesnt_beat_club() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Spades),
            &card(2, Suit::Clubs),
            Suit::Spades,
        ));
    }

    // Rule 2: kozyr beats everything except clubs and higher kozyr

    #[test]
    fn lowest_kozyr_beats_highest_other_suit() {
        assert!(Game::is_valid_play(
            &card(2, Suit::Spades),
            &card(14, Suit::Diamonds),
            Suit::Spades,
        ));
    }

    #[test]
    fn higher_kozyr_beats_lower_kozyr() {
        assert!(Game::is_valid_play(
            &card(10, Suit::Spades),
            &card(5, Suit::Spades),
            Suit::Spades,
        ));
    }

    #[test]
    fn lower_kozyr_doesnt_beat_higher_kozyr() {
        assert!(!Game::is_valid_play(
            &card(5, Suit::Spades),
            &card(10, Suit::Spades),
            Suit::Spades,
        ));
    }

    #[test]
    fn other_suit_doesnt_beat_kozyr() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Hearts),
            &card(2, Suit::Spades),
            Suit::Spades,
        ));
    }

    #[test]
    fn different_non_kozyr_suit_doesnt_beat() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Hearts),
            &card(2, Suit::Diamonds),
            Suit::Spades,
        ));
    }

    // Rule 3: if kozyr is clubs, there is no kozyr and all suits are equal

    #[test]
    fn clubs_kozyr_club_doesnt_beat_other_suit() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Clubs),
            &card(2, Suit::Hearts),
            Suit::Clubs,
        ));
    }

    #[test]
    fn clubs_kozyr_other_suit_doesnt_beat_club() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Hearts),
            &card(2, Suit::Clubs),
            Suit::Clubs,
        ));
    }

    #[test]
    fn clubs_kozyr_higher_club_beats_lower_club() {
        assert!(Game::is_valid_play(
            &card(10, Suit::Clubs),
            &card(5, Suit::Clubs),
            Suit::Clubs,
        ));
    }

    #[test]
    fn clubs_kozyr_same_suit_higher_beats() {
        assert!(Game::is_valid_play(
            &card(10, Suit::Hearts),
            &card(5, Suit::Hearts),
            Suit::Clubs,
        ));
    }

    #[test]
    fn clubs_kozyr_same_suit_lower_doesnt_beat() {
        assert!(!Game::is_valid_play(
            &card(5, Suit::Diamonds),
            &card(10, Suit::Diamonds),
            Suit::Clubs,
        ));
    }

    #[test]
    fn clubs_kozyr_different_suits_dont_beat() {
        assert!(!Game::is_valid_play(
            &card(14, Suit::Spades),
            &card(2, Suit::Hearts),
            Suit::Clubs,
        ));
    }
}
