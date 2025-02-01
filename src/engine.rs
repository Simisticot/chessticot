use rand::seq::IndexedRandom;

use crate::player::Player;
use crate::ChessMove;
use crate::Position;

pub struct FirstMovePlayer;

impl Player for FirstMovePlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        position.all_legal_moves().iter().next().unwrap().clone()
    }
}

pub struct RandomPlayer;

impl Player for RandomPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        position
            .all_legal_moves()
            .choose(&mut rand::rng())
            .unwrap()
            .clone()
    }
}
