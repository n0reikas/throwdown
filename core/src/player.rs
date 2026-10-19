use std::fmt;

use crate::card::Card;

#[derive(Debug)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub face_up_stack: Vec<Card>,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: usize, name: String) -> Self {
        Player {
            id,
            name,
            face_up_stack: Vec::new(),
            hand: Vec::new(),
        }
    }

    pub fn top_card(&self) -> Option<&Card> {
        self.face_up_stack.last()
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}'s hand:\n", &self.name.trim())?;
        for (i, card) in self.hand.iter().enumerate() {
            if i > 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}: {}", i + 1, card)?;
        }
        Ok(())
    }
}
