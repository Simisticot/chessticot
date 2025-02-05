use std::fmt::Display;

use crate::{ChessMove, Position};

pub trait Player: Display {
    fn offer_move(&self, position: &Position) -> ChessMove;
    fn evalutate(&self, position: &Position) -> isize;
}
