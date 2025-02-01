mod coords;
mod piece;

pub use crate::coords::{cards, eight_degrees, inter_cards, ChessMove, Coords, Direction, Move};
pub use crate::piece::{Piece, PieceColor, PieceKind};
use std::usize;

pub struct Game {
    pub current_position: Position,
    pub checkmated: Option<PieceColor>,
}

impl Game {
    pub fn start() -> Game {
        let mut board = Vec::new();
        for i in 0..8 {
            let mut row = Vec::new();
            for j in 0..8 {
                row.push(Piece::from_initial_position(j, i));
            }
            board.push(row);
        }
        Game {
            current_position: Position::initial(),
            checkmated: None,
        }
    }

    pub fn empty() -> Game {
        Game {
            current_position: Position::empty_board(),
            checkmated: None,
        }
    }
    pub fn make_move(&mut self, chess_move: &ChessMove) {
        if self.current_position.is_move_legal(chess_move) {
            self.current_position = self.current_position.after_move(chess_move);
            if self.current_position.is_checkmate() {
                self.checkmated = Some(self.current_position.to_move.clone());
            }
        }
    }
}

#[derive(Clone)]
pub struct Position {
    pub board: Vec<Vec<Option<Piece>>>,
    pub to_move: PieceColor,
    white_king_moved: bool,
    white_left_rook_moved: bool,
    white_right_rook_moved: bool,
    black_king_moved: bool,
    black_left_rook_moved: bool,
    black_right_rook_moved: bool,
    en_passant_on: Option<Coords>,
}

impl Position {
    pub fn initial() -> Position {
        let mut board = Vec::new();
        for i in 0..8 {
            let mut row = Vec::new();
            for j in 0..8 {
                row.push(Piece::from_initial_position(j, i));
            }
            board.push(row);
        }
        Position {
            board,
            to_move: PieceColor::White,
            white_king_moved: false,
            white_left_rook_moved: false,
            white_right_rook_moved: false,
            black_king_moved: false,
            black_left_rook_moved: false,
            black_right_rook_moved: false,
            en_passant_on: None,
        }
    }
    pub fn empty_board() -> Position {
        let mut board = Vec::new();
        for _ in 0..8 {
            let mut row = Vec::new();
            for _ in 0..8 {
                row.push(None);
            }
            board.push(row);
        }
        Position {
            board,
            to_move: PieceColor::White,
            white_king_moved: false,
            white_left_rook_moved: false,
            white_right_rook_moved: false,
            black_king_moved: false,
            black_left_rook_moved: false,
            black_right_rook_moved: false,
            en_passant_on: None,
        }
    }
    pub fn opposite_color_to_move(&self) -> Position {
        let mut new_position = self.clone();
        new_position.to_move = new_position.to_move.opposite();
        new_position
    }

    pub fn color_to_move(&self, color: PieceColor) -> Position {
        Position {
            to_move: color,
            ..self.clone()
        }
    }

    pub fn after_move(&self, chess_move: &ChessMove) -> Position {
        let mut new_board = self.board.clone();
        let mut en_passant_on = None;
        match chess_move {
            ChessMove::RegularMove(coordinates) => {
                move_piece(&mut new_board, coordinates.origin, coordinates.destination);
            }
            ChessMove::PawnSkip(movement) => {
                move_piece(&mut new_board, movement.origin, movement.destination);
                en_passant_on = Some(Coords {
                    x: movement.origin.x,
                    y: (movement.origin.y + movement.destination.y) / 2 as isize,
                });
            }
            ChessMove::CastleLeft => {
                let row = self.to_move.homerow();
                move_piece(
                    &mut new_board,
                    Coords { x: 4, y: row },
                    Coords { x: 2, y: row },
                );
                move_piece(
                    &mut new_board,
                    Coords { x: 0, y: row },
                    Coords { x: 3, y: row },
                );
            }
            ChessMove::CastleRight => {
                let row = self.to_move.homerow();
                move_piece(
                    &mut new_board,
                    Coords { x: 4, y: row },
                    Coords { x: 6, y: row },
                );
                move_piece(
                    &mut new_board,
                    Coords { x: 7, y: row },
                    Coords { x: 5, y: row },
                );
            }
            ChessMove::EnPassant(movement, pawn_taken) => {
                move_piece(&mut new_board, movement.origin, movement.destination);
                take_piece_at(&mut new_board, *pawn_taken);
            }
            ChessMove::Promotion(movement, promoted_to) => {
                take_piece_at(&mut new_board, movement.origin);
                put_piece_at(
                    &mut new_board,
                    Piece {
                        kind: *promoted_to,
                        color: self.to_move.clone(),
                    },
                    movement.destination,
                );
            }
        }
        Position {
            board: new_board,
            to_move: self.to_move.opposite(),
            en_passant_on,
            ..self.clone()
        }
    }
    pub fn is_checkmate(&self) -> bool {
        return self.is_in_check(&self.to_move) && self.all_legal_moves().len() == 0;
    }
    pub fn all_legal_moves(&self) -> Vec<ChessMove> {
        all_squares()
            .iter()
            .map(|square| self.legal_moves_from_origin(square))
            .flatten()
            .collect()
    }

    fn all_possible_moves(&self) -> Vec<ChessMove> {
        all_squares()
            .iter()
            .map(|square| self.possible_moves_from_origin(square))
            .flatten()
            .collect()
    }
    pub fn legal_moves_from_origin(&self, origin: &Coords) -> Vec<ChessMove> {
        self.possible_moves_from_origin(origin)
            .iter()
            .cloned()
            .filter(|chess_move| !self.opens_own_king(chess_move))
            .collect()
    }

    fn possible_moves_from_origin(&self, origin: &Coords) -> Vec<ChessMove> {
        match piece_at(&self.board, origin) {
            None => Vec::new(),
            Some(piece) => {
                if piece.color == self.to_move {
                    self.movement_from_origin(origin, piece)
                } else {
                    Vec::new()
                }
            }
        }
    }
    fn is_move_legal(&self, chess_move: &ChessMove) -> bool {
        let origin = match chess_move {
            ChessMove::RegularMove(movement) => movement.origin,
            ChessMove::PawnSkip(movement) => movement.origin,
            ChessMove::EnPassant(movement, _) => movement.origin,
            ChessMove::CastleRight | ChessMove::CastleLeft => {
                let row = match self.to_move {
                    PieceColor::White => 0,
                    PieceColor::Black => 7,
                };
                Coords { y: row, x: 4 }
            }
            ChessMove::Promotion(movement, _) => movement.origin,
        };

        self.legal_moves_from_origin(&origin).contains(chess_move)
    }
    fn is_attacked_by(&self, by: &PieceColor, square: &Coords) -> bool {
        self.color_to_move(by.clone())
            .all_possible_moves()
            .iter()
            .map(|chess_move| match chess_move {
                ChessMove::RegularMove(movement) => &movement.destination == square,
                ChessMove::Promotion(movement, _) => &movement.destination == square,
                ChessMove::EnPassant(_, taken) => taken == square,
                _ => false,
            })
            .any(|attacks_square| attacks_square)
    }
    fn is_in_check(&self, color: &PieceColor) -> bool {
        match self.king_location(color) {
            None => false,
            Some(loc) => self.is_attacked_by(&color.opposite(), &loc),
        }
    }
    fn opens_own_king(&self, chess_move: &ChessMove) -> bool {
        let potential_position = self.after_move(chess_move);
        potential_position.is_in_check(&self.to_move)
    }

    pub fn king_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_king_moved,
            PieceColor::Black => &self.black_king_moved,
        }
    }

    pub fn right_rook_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_right_rook_moved,
            PieceColor::Black => &self.black_right_rook_moved,
        }
    }

    pub fn left_rook_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_left_rook_moved,
            PieceColor::Black => &self.black_left_rook_moved,
        }
    }
    fn movement_from_origin(&self, origin: &Coords, piece: Piece) -> Vec<ChessMove> {
        match piece.kind {
            PieceKind::Pawn => self.pawn_from(origin, &piece.color),
            PieceKind::Rook => self.rook_from(origin, &piece.color),
            PieceKind::Knight => self.knight_from(origin, &piece.color),
            PieceKind::Bishop => self.bishop_from(origin, &piece.color),
            PieceKind::Queen => self.queen_movement(origin, &piece.color),
            PieceKind::King => self.king_movement(origin, &piece.color),
        }
    }
    fn king_movement(&self, origin: &Coords, origin_color: &PieceColor) -> Vec<ChessMove> {
        let mut moves =
            projected_movement(&self.board, origin, eight_degrees(), origin_color, Some(1));
        let row = origin_color.homerow();
        if piece_at(&self.board, &Coords { y: row, x: 5 }).is_none()
            && piece_at(&self.board, &Coords { y: row, x: 6 }).is_none()
            && piece_at(&self.board, &Coords { y: row, x: 4 }).is_some_and(|piece| {
                piece
                    == Piece {
                        kind: PieceKind::King,
                        color: origin_color.clone(),
                    }
            })
            && !self.king_moved(origin_color)
            && piece_at(&self.board, &Coords { y: row, x: 7 }).is_some_and(|piece| {
                piece
                    == Piece {
                        kind: PieceKind::Rook,
                        color: origin_color.clone(),
                    }
            })
            && !self.right_rook_moved(origin_color)
        {
            moves.push(ChessMove::CastleRight);
        }
        if piece_at(&self.board, &Coords { y: row, x: 3 }).is_none()
            && piece_at(&self.board, &Coords { y: row, x: 2 }).is_none()
            && piece_at(&self.board, &Coords { y: row, x: 1 }).is_none()
            && piece_at(&self.board, &Coords { y: row, x: 4 }).is_some_and(|piece| {
                piece
                    == Piece {
                        kind: PieceKind::King,
                        color: origin_color.clone(),
                    }
            })
            && !self.king_moved(origin_color)
            && piece_at(&self.board, &Coords { y: row, x: 0 }).is_some_and(|piece| {
                piece
                    == Piece {
                        kind: PieceKind::Rook,
                        color: origin_color.clone(),
                    }
            })
            && !self.left_rook_moved(origin_color)
        {
            moves.push(ChessMove::CastleLeft);
        }

        moves
    }
    fn queen_movement(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        projected_movement(&self.board, origin, eight_degrees(), color, None)
    }
    fn bishop_from(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        projected_movement(&self.board, origin, inter_cards(), color, None)
    }
    fn knight_from(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        let directions: Vec<Direction> = vec![
            Direction { dy: 2, dx: 1 },
            Direction { dy: 2, dx: -1 },
            Direction { dy: 1, dx: 2 },
            Direction { dy: 1, dx: -2 },
            Direction { dy: -2, dx: 1 },
            Direction { dy: -2, dx: -1 },
            Direction { dy: -1, dx: -2 },
            Direction { dy: -1, dx: 2 },
        ];
        let potential_moves = directions.iter().map(|direction| {
            ChessMove::RegularMove(Move {
                origin: origin.clone(),
                destination: *origin + *direction,
            })
        });
        potential_moves
            .into_iter()
            .filter(|chess_move| match chess_move {
                ChessMove::RegularMove(coordinates) => {
                    coordinates.destination.is_in_bounds()
                        && piece_at(&self.board, &coordinates.destination)
                            .is_none_or(|piece| &piece.color != color)
                }
                _ => false,
            })
            .collect()
    }
    fn rook_from(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        projected_movement(&self.board, origin, cards(), color, None)
    }
    fn pawn_from(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        let mut legal_moves = vec![];
        let forward = Direction {
            dx: 0,
            dy: color.pawn_orientation(),
        };
        let ahead_one = *origin + forward;
        let ahead_two = ahead_one + forward;

        if !ahead_one.is_in_bounds() {
            return legal_moves;
        }

        if piece_at(&self.board, &ahead_one).is_none() {
            legal_moves.push(ChessMove::RegularMove(Move {
                origin: origin.clone(),
                destination: ahead_one,
            }));
            if ahead_two.is_in_bounds()
                && (origin.y == 1 || origin.y == 6)
                && piece_at(&self.board, &ahead_two).is_none()
            {
                legal_moves.push(ChessMove::PawnSkip(Move {
                    origin: origin.clone(),
                    destination: ahead_two,
                }));
            }
        }

        vec![
            Coords {
                x: origin.x + 1,
                y: origin.y + color.pawn_orientation(),
            },
            Coords {
                x: origin.x - 1,
                y: origin.y + color.pawn_orientation(),
            },
        ]
        .iter()
        .for_each(|diagonal| {
            if diagonal.is_in_bounds() {
                match piece_at(&self.board, &diagonal) {
                    None => {}
                    Some(piece) => {
                        if piece.color == color.opposite() {
                            legal_moves.push(ChessMove::RegularMove(Move {
                                origin: origin.clone(),
                                destination: *diagonal,
                            }));
                        }
                    }
                }
            }
        });
        if let Some(en_passant) = self.en_passant_from(origin, color) {
            legal_moves.push(en_passant);
        }
        legal_moves
            .iter()
            .map(|pawn_move| match pawn_move {
                ChessMove::RegularMove(movement) => {
                    if movement.destination.y == color.opposite().homerow() {
                        PieceKind::promoteable()
                            .map(|promotable_kind| {
                                ChessMove::Promotion(movement.clone(), promotable_kind.clone())
                            })
                            .collect()
                    } else {
                        vec![pawn_move.clone()]
                    }
                }
                ChessMove::PawnSkip(_) => vec![pawn_move.clone()],
                ChessMove::EnPassant(_, _) => vec![pawn_move.clone()],
                _ => panic!("Pawn moves should only be regular, skip or en passant"),
            })
            .flatten()
            .collect()
    }
    fn en_passant_from(&self, origin: &Coords, color: &PieceColor) -> Option<ChessMove> {
        match self.en_passant_on {
            None => None,
            Some(coordinates) => {
                for candidate in vec![
                    coordinates
                        + Direction {
                            dx: 1,
                            dy: color.opposite().pawn_orientation(),
                        },
                    coordinates
                        + Direction {
                            dx: -1,
                            dy: color.opposite().pawn_orientation(),
                        },
                ] {
                    if candidate.is_in_bounds() && candidate == *origin {
                        return Some(ChessMove::EnPassant(
                            Move {
                                origin: origin.clone(),
                                destination: coordinates.clone(),
                            },
                            coordinates
                                + Direction {
                                    dx: 0,
                                    dy: color.opposite().pawn_orientation(),
                                },
                        ));
                    }
                }
                None
            }
        }
    }
    fn king_location(&self, color: &PieceColor) -> Option<Coords> {
        for i in 0..8 {
            for j in 0..8 {
                let loc = Coords { y: i, x: j };
                if piece_at(&self.board, &loc)
                    .is_some_and(|piece| piece.kind == PieceKind::King && piece.color == *color)
                {
                    return Some(loc);
                }
            }
        }
        None
    }
}

fn projected_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    directions: Vec<Direction>,
    origin_color: &PieceColor,
    limit: Option<isize>,
) -> Vec<ChessMove> {
    directions
        .iter()
        .map(|dir| raycast(board, origin, dir, origin_color, limit))
        .flatten()
        .map(|destination| {
            ChessMove::RegularMove(Move {
                origin: origin.clone(),
                destination,
            })
        })
        .collect()
}
pub fn raycast(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    direction: &Direction,
    origin_color: &PieceColor,
    limit: Option<isize>,
) -> Vec<Coords> {
    let limit = limit.unwrap_or(7) + 1;
    let mut squares = vec![];
    // for instead of loop to avoid potential infinite loop
    for i in 1..limit {
        let next_square = *origin + (*direction * i);
        if !next_square.is_in_bounds() {
            break;
        }
        if let Some(piece) = piece_at(board, &next_square) {
            if piece.color == origin_color.opposite() {
                squares.push(next_square);
            }
            break;
        }
        squares.push(next_square);
    }
    squares
}

fn all_squares() -> Vec<Coords> {
    let mut squares = Vec::new();
    for i in 0..8 {
        for j in 0..8 {
            squares.push(Coords { y: i, x: j });
        }
    }
    squares
}

fn move_piece(board: &mut Vec<Vec<Option<Piece>>>, origin: Coords, dest: Coords) {
    if let Some(origin_piece) = take_piece_at(board, origin) {
        put_piece_at(board, origin_piece, dest);
    }
}
pub fn piece_at(board: &Vec<Vec<Option<Piece>>>, loc: &Coords) -> Option<Piece> {
    board[loc.y as usize][loc.x as usize].clone()
}
pub fn take_piece_at(board: &mut Vec<Vec<Option<Piece>>>, loc: Coords) -> Option<Piece> {
    board[loc.y as usize][loc.x as usize].take()
}
pub fn put_piece_at(board: &mut Vec<Vec<Option<Piece>>>, piece: Piece, loc: Coords) {
    board[loc.y as usize][loc.x as usize] = Some(piece);
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{collections::HashSet, hash::RandomState};

    use super::*;

    #[test]
    fn pawn_homerow() {
        let mut position = Position::empty_board();
        position.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: Coords { y: 2, x: 4 }
                }),
                ChessMove::PawnSkip(Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                })
            ]
        )
    }

    #[test]
    fn pawn_not_homerow() {
        let mut position = Position::empty_board();
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: Coords { y: 3, x: 4 }
            })]
        )
    }

    #[test]
    fn pawn_not_homerow_with_capture() {
        let mut position = Position::empty_board();
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                }),
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: opposing_pawn_location
                })
            ]
        )
    }

    #[test]
    fn pawn_not_homerow_blocked() {
        let mut position = Position::empty_board();
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(position.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_edge_of_board_horizontal_blocked() {
        let mut position = Position::empty_board();
        position.board[2][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 7 };
        assert_eq!(position.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_edge_of_board_vertical() {
        let mut position = Position::empty_board();
        position.board[7][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 7, x: 4 };
        assert_eq!(position.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_not_homerow_with_capture_blocked() {
        let mut position = Position::empty_board();
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        position.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: opposing_pawn_location
            })]
        )
    }

    #[test]
    fn pawn_homerow_blocked() {
        let mut position = Position::empty_board();
        position.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(position.legal_moves_from_origin(&pawn_location), vec![])
    }
    #[test]
    fn pawn_homerow_second_square_blocked() {
        let mut position = Position::empty_board();
        position.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: Coords { y: 2, x: 4 }
            })]
        )
    }

    #[test]
    fn pawn_homerow_with_capture_blocked() {
        let mut position = Position::empty_board();
        position.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        position.board[2][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        let capture_location = Coords { y: 2, x: 5 };
        assert_eq!(
            position.legal_moves_from_origin(&pawn_location),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: capture_location
            })]
        )
    }

    #[test]
    fn rook_middle_board() {
        let mut position = Position::empty_board();
        position.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let mut legal_moves = vec![];

        for j in 0..8 {
            if j != 4 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: rook_location,
                    destination: Coords { y: 4, x: j },
                }));
            }
        }
        for i in 0..8 {
            if i != 4 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: rook_location,
                    destination: Coords { x: 4, y: i },
                }));
            }
        }

        let legal_move_set: HashSet<ChessMove, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_moves: HashSet<ChessMove, RandomState> =
            HashSet::from_iter(position.legal_moves_from_origin(&rook_location));
        let diff: HashSet<&ChessMove, RandomState> =
            legal_move_set.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn rook_middle_board_boxed_in_opposite_color() {
        let mut position = Position::empty_board();
        position.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        position.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        position.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        position.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let up = Coords { y: 5, x: 4 };
        let down = Coords { y: 3, x: 4 };
        let left = Coords { y: 4, x: 3 };
        let right = Coords { y: 4, x: 5 };

        let legal_moves = vec![
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: up,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: down,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: left,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: right,
            }),
        ];

        assert_eq!(
            position.legal_moves_from_origin(&rook_location),
            legal_moves
        );
    }

    #[test]
    fn rook_middle_board_boxed_in_own_color() {
        let mut position = Position::empty_board();
        position.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };

        let legal_moves = vec![];

        assert_eq!(
            position.legal_moves_from_origin(&rook_location),
            legal_moves
        );
    }

    #[test]
    fn knight_middle_board() {
        let mut position = Position::empty_board();
        position.board[3][3] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 3, x: 3 };

        let legal_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            vec![
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 4 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 5 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 2 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 1 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 4 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 5 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                }),
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            position
                .legal_moves_from_origin(&knight_location)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn knight_corner() {
        let mut position = Position::empty_board();
        position.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        let legal_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            vec![
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                }),
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            position
                .legal_moves_from_origin(&knight_location)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn knight_corner_blocked() {
        let mut position = Position::empty_board();
        position.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        position.board[1][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        position.board[2][1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        assert_eq!(position.legal_moves_from_origin(&knight_location).len(), 0)
    }

    #[test]
    fn bishob_middle_board() {
        let mut position = Position::empty_board();
        position.board[3][3] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::White,
        });
        let bishop_location = Coords { y: 3, x: 3 };
        let mut legal_moves = vec![];

        for j in 0..8 {
            if j != 3 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: bishop_location,
                    destination: Coords { y: j, x: j },
                }));
            }
        }

        for i in 0..7 {
            if i != 3 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: bishop_location,
                    destination: Coords { y: 6 - i, x: i },
                }));
            }
        }

        let legal_move_set: HashSet<ChessMove, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_move_set: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            position
                .legal_moves_from_origin(&bishop_location)
                .iter()
                .cloned(),
        );
        let difference_set: HashSet<&ChessMove, RandomState> = legal_move_set
            .symmetric_difference(&found_move_set)
            .collect();
        assert_eq!(difference_set, HashSet::new());
    }

    #[test]
    fn king_middle_board() {
        let mut position = Position::empty_board();
        position.board[3][3] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        let king_location = Coords { y: 3, x: 3 };
        let legal_moves = HashSet::from([
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 4 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 3 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 3 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 4 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 4 },
            }),
        ]);

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            position
                .legal_moves_from_origin(&king_location)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn cannot_move_out_of_turn() {
        let mut position = Position::empty_board();
        position.board[3][3] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 3, x: 3 };
        assert_eq!(position.legal_moves_from_origin(&king_location).len(), 0);
    }

    #[test]
    fn execute_move_into_check() {
        let mut position = Position::empty_board();

        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        let king_destination = Coords { y: 0, x: 1 };

        let new_position = position.after_move(&ChessMove::RegularMove(Move {
            origin: king_location,
            destination: king_destination,
        }));
        assert!(new_position.king_location(&PieceColor::White) == Some(king_destination.clone()));
        assert!(new_position.is_attacked_by(&PieceColor::Black, &king_destination,));
        assert!(new_position.is_in_check(&PieceColor::White));
    }

    #[test]
    fn detects_move_into_check() {
        let mut position = Position::empty_board();

        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        assert!(position.opens_own_king(&ChessMove::RegularMove(Move {
            origin: king_location,
            destination: Coords { y: 0, x: 1 },
        }),));
    }

    #[test]
    fn cannot_move_into_check() {
        let mut position = Position::empty_board();

        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        assert!(!position.is_move_legal(&ChessMove::RegularMove(Move {
            origin: king_location,
            destination: Coords { y: 0, x: 1 },
        }),));
    }

    #[test]
    fn detects_check() {
        let mut position = Position::empty_board();

        position.board[0][1] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        assert!(position.is_in_check(&PieceColor::White));
    }

    #[test]
    fn detects_attacked() {
        let mut position = Position::empty_board();

        position.board[0][1] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 1 };
        assert!(position.is_attacked_by(&PieceColor::Black, &king_location,));
    }

    #[test]
    fn detects_checkmate() {
        let mut position = Position::empty_board();

        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[1][1] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::Black,
        });
        position.board[2][2] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::Black,
        });
        assert!(position.is_checkmate());
    }

    #[test]
    fn finds_king() {
        let mut position = Position::empty_board();

        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        assert_eq!(
            position.king_location(&PieceColor::White).unwrap(),
            Coords { x: 0, y: 0 }
        )
    }

    #[test]
    fn can_castle_right() {
        let mut position = Position::empty_board();

        position.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        assert!(
            piece_at(&position.board, &Coords { y: 0, x: 5 }).is_none()
                && piece_at(&position.board, &Coords { y: 0, x: 6 }).is_none()
        );
        assert!(position
            .legal_moves_from_origin(&Coords { y: 0, x: 4 })
            .contains(&ChessMove::CastleRight));
        assert!(position.is_move_legal(&ChessMove::CastleRight,))
    }

    #[test]
    fn castle_right() {
        let mut game = Game::empty();

        game.current_position.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.current_position.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.make_move(&ChessMove::CastleRight);
        assert!(piece_at(&game.current_position.board, &Coords { y: 0, x: 4 }).is_none());
        assert!(piece_at(&game.current_position.board, &Coords { y: 0, x: 7 }).is_none());
        assert!(
            piece_at(&game.current_position.board, &Coords { y: 0, x: 6 }).is_some_and(
                |piece| piece
                    == Piece {
                        kind: PieceKind::King,
                        color: PieceColor::White
                    }
            )
        );
        assert!(
            piece_at(&game.current_position.board, &Coords { y: 0, x: 5 }).is_some_and(
                |piece| piece
                    == Piece {
                        kind: PieceKind::Rook,
                        color: PieceColor::White
                    }
            )
        );
    }

    #[test]
    fn make_move() {
        let mut game = Game::empty();

        game.current_position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        let king_location = Coords { x: 0, y: 0 };
        game.make_move(&ChessMove::RegularMove(Move {
            origin: king_location,
            destination: Coords { x: 0, y: 1 },
        }));
        assert!(piece_at(&game.current_position.board, &king_location).is_none());
        assert_eq!(
            piece_at(&game.current_position.board, &Coords { x: 0, y: 1 })
                .unwrap()
                .kind,
            PieceKind::King
        );
    }

    #[test]
    fn scholars_mate() {
        let mut game = Game::start();

        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { x: 4, y: 1 },
            destination: Coords { x: 4, y: 3 },
        }));
        assert!(game.current_position.to_move == PieceColor::Black);
        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { x: 4, y: 6 },
            destination: Coords { x: 4, y: 4 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 3, y: 0 },
            destination: Coords { x: 7, y: 4 },
        }));
        assert!(game.current_position.to_move == PieceColor::Black);
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 1, y: 7 },
            destination: Coords { x: 2, y: 5 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 5, y: 0 },
            destination: Coords { x: 2, y: 3 },
        }));
        assert!(game.current_position.to_move == PieceColor::Black);
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 6, y: 7 },
            destination: Coords { x: 5, y: 5 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 7, y: 4 },
            destination: Coords { x: 5, y: 6 },
        }));
        assert!(game.current_position.to_move == PieceColor::Black);

        assert!(game.checkmated == Some(PieceColor::Black));
    }

    #[test]
    fn pawn_skip_is_legal() {
        let position = Position::initial();
        assert!(position.is_move_legal(&ChessMove::PawnSkip(Move {
            origin: Coords { x: 4, y: 1 },
            destination: Coords { x: 4, y: 3 }
        }),))
    }

    #[test]
    fn en_passant_right() {
        let mut position = Position::empty_board();
        position.board[1][1] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][0] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let after_skip = position.after_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 1 },
            destination: Coords { y: 3, x: 1 },
        }));
        let black_pawn_location = Coords { y: 3, x: 0 };
        let ep = ChessMove::EnPassant(
            Move {
                origin: black_pawn_location,
                destination: Coords { y: 2, x: 1 },
            },
            Coords { y: 3, x: 1 },
        );
        assert!(after_skip.en_passant_on == Some(Coords { y: 2, x: 1 }));
        assert!(after_skip
            .legal_moves_from_origin(&black_pawn_location)
            .contains(&ep));
        assert!(after_skip.is_move_legal(&ep))
    }
    #[test]
    fn en_passant_left() {
        let mut position = Position::empty_board();
        position.board[1][1] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[3][2] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let after_skip = position.after_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 1 },
            destination: Coords { y: 3, x: 1 },
        }));
        let black_pawn_location = Coords { y: 3, x: 2 };
        let ep = ChessMove::EnPassant(
            Move {
                origin: black_pawn_location,
                destination: Coords { y: 2, x: 1 },
            },
            Coords { y: 3, x: 1 },
        );
        assert!(after_skip.en_passant_on == Some(Coords { y: 2, x: 1 }));
        assert!(after_skip
            .legal_moves_from_origin(&black_pawn_location)
            .contains(&ep));
        assert!(after_skip.is_move_legal(&ep))
    }
    #[test]
    fn no_en_passant_from_accross_the_board() {
        let mut position = Position::empty_board();
        position.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        position.board[7][2] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let after_skip = position.after_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 4 },
            destination: Coords { y: 3, x: 4 },
        }));

        assert!(after_skip.en_passant_on == Some(Coords { y: 2, x: 4 }));
        assert!(!after_skip.is_move_legal(&ChessMove::EnPassant(
            Move {
                origin: Coords { y: 7, x: 2 },
                destination: Coords { y: 2, x: 4 }
            },
            Coords { y: 3, x: 4 }
        ),))
    }
    #[test]
    fn promotion() {
        let mut position = Position::empty_board();
        position.board[6][0] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_position = Coords { y: 6, x: 0 };
        dbg!(position.legal_moves_from_origin(&pawn_position));
        assert_eq!(PieceColor::White.opposite().homerow(), 7 as isize);
        position
            .legal_moves_from_origin(&pawn_position)
            .iter()
            .for_each(|chess_move| match chess_move {
                ChessMove::Promotion(_, _) => (),
                _ => panic!("expected only promotions, found {:?}", chess_move),
            });
    }
    #[test]
    fn promotion_is_an_attack() {
        let mut position = Position::empty_board();
        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[1][1] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        position
            .color_to_move(PieceColor::Black)
            .all_possible_moves()
            .iter()
            .for_each(|chess_move| match chess_move {
                ChessMove::Promotion(_, _) => (),
                _ => panic!("only promotions in this position, found {:?}", chess_move),
            });
        assert!(position.is_attacked_by(&PieceColor::Black, &king_location,));
    }
}
