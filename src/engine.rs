use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::fmt::Display;
use std::isize;

use crate::all_squares;
use crate::piece_at;
use crate::player::Player;
use crate::ChessMove;
use crate::Piece;
use crate::PieceColor;
use crate::PieceKind;
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

pub struct BasicEvaluationPlayer;

impl BasicEvaluationPlayer {
    fn evaluate(&self, position: &Position) -> isize {
        all_squares()
            .iter()
            .map(|square| match piece_at(&position.board, square) {
                None => 0 as isize,
                Some(piece) => self.evaluate_piece(
                    &piece,
                    position.is_attacked_by(&position.to_move.opposite(), square),
                    &position.to_move,
                ),
            })
            .reduce(|acc, e| acc + e)
            .expect("all squares is never 0 length")
    }

    fn evaluate_piece(&self, piece: &Piece, is_attacked: bool, to_move: &PieceColor) -> isize {
        self.piece_value(&piece.kind)
            * (if to_move == &piece.color { 1 } else { -1 } * {
                if !is_attacked {
                    2
                } else {
                    1
                }
            })
    }
    fn piece_value(&self, kind: &PieceKind) -> isize {
        match kind {
            PieceKind::King => 0,
            PieceKind::Pawn => 10,
            PieceKind::Rook => 50,
            PieceKind::Bishop => 30,
            PieceKind::Knight => 20,
            PieceKind::Queen => 100,
        }
    }
}

impl Display for BasicEvaluationPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Basic Evaluation")
    }
}

impl Player for BasicEvaluationPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        let all_moves = position.all_legal_moves();
        let mut moves_by_evaluation = HashMap::new();
        all_moves.iter().for_each(|chess_move| {
            moves_by_evaluation
                .entry(self.evaluate(position))
                .or_insert(Vec::new())
                .push(chess_move.clone())
        });
        moves_by_evaluation
            .get(moves_by_evaluation.keys().max().unwrap())
            .unwrap()
            .choose(&mut rand::rng())
            .unwrap()
            .clone()
    }
}
