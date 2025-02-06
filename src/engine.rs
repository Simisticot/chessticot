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
    fn evalutate(&self, _position: &Position) -> isize {
        0
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
    fn evalutate(&self, _position: &Position) -> isize {
        0
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
    fn evalutate(&self, _position: &Position) -> isize {
        0
    }
}

fn basic_evaluation(position: &Position) -> isize {
    fn piece_value(kind: &PieceKind) -> isize {
        match kind {
            PieceKind::King => 0,
            PieceKind::Pawn => 10,
            PieceKind::Rook => 50,
            PieceKind::Bishop => 30,
            PieceKind::Knight => 20,
            PieceKind::Queen => 100,
        }
    }
    fn evaluate_piece(piece: &Piece, is_attacked: bool, to_move: &PieceColor) -> isize {
        piece_value(&piece.kind)
            * (if to_move == &piece.color { 1 } else { -1 } * {
                if !is_attacked {
                    2
                } else {
                    1
                }
            })
    }
    all_squares()
        .iter()
        .map(|square| match piece_at(&position.board, square) {
            None => 0 as isize,
            Some(piece) => evaluate_piece(
                &piece,
                position.is_attacked_by(&position.to_move.opposite(), square),
                &position.to_move,
            ),
        })
        .reduce(|acc, e| acc + e)
        .expect("all squares is never 0 length")
}
pub struct BasicEvaluationPlayer;

impl Display for BasicEvaluationPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Basic Evaluation")
    }
}

fn moves_with_evaluation(
    position: &Position,
    evaluation: fn(&Position) -> isize,
) -> HashMap<isize, Vec<ChessMove>> {
    let all_moves = position.all_legal_moves();
    let mut moves_by_evaluation = HashMap::new();
    all_moves.iter().for_each(|chess_move| {
        moves_by_evaluation
            .entry(evaluation(&position.after_move(chess_move)))
            .or_insert(Vec::new())
            .push(chess_move.clone())
    });
    moves_by_evaluation
}

fn random_move_with_max_evaluation(
    moves_by_evaluation: HashMap<isize, Vec<ChessMove>>,
) -> ChessMove {
    moves_by_evaluation
        .get(moves_by_evaluation.keys().max().unwrap())
        .unwrap()
        .choose(&mut rand::rng())
        .unwrap()
        .clone()
}

fn random_move_with_min_evaluation(
    moves_by_evaluation: HashMap<isize, Vec<ChessMove>>,
) -> ChessMove {
    moves_by_evaluation
        .get(moves_by_evaluation.keys().min().unwrap())
        .unwrap()
        .choose(&mut rand::rng())
        .unwrap()
        .clone()
}

impl Player for BasicEvaluationPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        random_move_with_max_evaluation(moves_with_evaluation(position, basic_evaluation))
    }
    fn evalutate(&self, position: &Position) -> isize {
        basic_evaluation(position)
    }
}

pub struct BetterEvaluationPlayer {}

impl Player for BetterEvaluationPlayer {
    fn offer_move(&self, position: &Position) -> ChessMove {
        random_move_with_max_evaluation(moves_with_evaluation(position, better_evaluation))
    }
    fn evalutate(&self, position: &Position) -> isize {
        better_evaluation(position)
    }
}

impl Display for BetterEvaluationPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Better evaluation")
    }
}

fn better_evaluation(position: &Position) -> isize {
    fn piece_value(kind: &PieceKind) -> isize {
        match kind {
            PieceKind::King => 1000,
            PieceKind::Pawn => 10,
            PieceKind::Rook => 50,
            PieceKind::Bishop => 30,
            PieceKind::Knight => 20,
            PieceKind::Queen => 500,
        }
    }
    fn evaluate_piece(
        piece: &Piece,
        is_attacked: bool,
        to_move: &PieceColor,
        controlled_squares: isize,
    ) -> isize {
        let value = piece_value(&piece.kind);
        let control_value = 2;
        let attacked_factor = if is_attacked {
            if &piece.color == to_move {
                -5
            } else {
                -piece_value(&piece.kind)
            }
        } else {
            0
        };
        (value + (controlled_squares * control_value)) + attacked_factor
    }
    let score_from_all_squares = all_squares()
        .iter()
        .map(|square| match piece_at(&position.board, square) {
            None => 0 as isize,
            Some(piece) => evaluate_piece(
                &piece,
                position.is_attacked_by(&position.to_move.opposite(), square),
                &position.to_move,
                position
                    .color_to_move(piece.color)
                    .legal_moves_from_origin(square)
                    .len()
                    .try_into()
                    .unwrap(),
            ),
        })
        .reduce(|acc, e| acc + e)
        .expect("all squares is never 0 length");
    let score_from_checkmate = if position.is_checkmate() { 10000000 } else { 0 };
    score_from_all_squares + score_from_checkmate
}
