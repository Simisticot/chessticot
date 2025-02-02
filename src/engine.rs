use rand::seq::IndexedRandom;
use std::fmt::Display;

use crate::player::Player;
use crate::ChessMove;
use crate::Position;

pub struct FirstMovePlayer;

impl Display for FirstMovePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "First available move")
    }
}
impl Player for FirstMovePlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        position.all_legal_moves().iter().next().unwrap().clone()
    }
}

pub struct RandomPlayer;

impl Display for RandomPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Random")
    }
}

impl Player for RandomPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        pick_random_move(position)
    }
}

fn pick_random_move(position: &Position) -> ChessMove {
    position
        .all_legal_moves()
        .choose(&mut rand::rng())
        .unwrap()
        .clone()
}

pub struct RandomCapturePrioPlayer;

impl Display for RandomCapturePrioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prioritize Capture")
    }
}

impl Player for RandomCapturePrioPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        let moves_with_capture: Vec<ChessMove> = position
            .all_legal_moves()
            .into_iter()
            .filter(|chess_move| {
                position
                    .after_move(chess_move)
                    .piece_count(position.to_move.opposite())
                    < position.piece_count(position.to_move.opposite())
            })
            .collect();
        if moves_with_capture.len() > 0 {
            moves_with_capture
                .choose(&mut rand::rng())
                .unwrap()
                .clone()
                .clone()
        } else {
            pick_random_move(position)
        }
    }
}
