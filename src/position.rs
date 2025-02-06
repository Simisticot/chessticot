use std::str;

use crate::all_squares;
use crate::cards;
use crate::eight_degrees;
use crate::inter_cards;
use crate::move_piece;
use crate::piece_at;
use crate::put_piece_at;
use crate::take_piece_at;
use crate::ChessMove;
use crate::Coords;
use crate::Direction;
use crate::Move;
use crate::Piece;
use crate::PieceColor;
use crate::PieceKind;

#[derive(Clone, Debug, PartialEq)]
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
    pub fn from_fen(fen_record: &str) -> Position {
        let fields: Vec<&str> = fen_record.split(" ").collect();

        assert!(fields.len() == 6);

        let mut board = vec![vec![]; 8];
        let mut rank = 7;
        fields[0].chars().for_each(|character| match character {
            '1'..='8' => {
                for _ in 0..character.to_digit(10).expect("matched digits 1 through 8") {
                    board[rank].push(None);
                }
            }
            '/' => {
                rank -= 1;
            }
            'r' => board[rank].push(Some(Piece {
                kind: PieceKind::Rook,
                color: PieceColor::Black,
            })),
            'n' => board[rank].push(Some(Piece {
                kind: PieceKind::Knight,
                color: PieceColor::Black,
            })),
            'b' => board[rank].push(Some(Piece {
                kind: PieceKind::Bishop,
                color: PieceColor::Black,
            })),
            'q' => board[rank].push(Some(Piece {
                kind: PieceKind::Queen,
                color: PieceColor::Black,
            })),
            'k' => board[rank].push(Some(Piece {
                kind: PieceKind::King,
                color: PieceColor::Black,
            })),
            'p' => board[rank].push(Some(Piece {
                kind: PieceKind::Pawn,
                color: PieceColor::Black,
            })),
            'R' => board[rank].push(Some(Piece {
                kind: PieceKind::Rook,
                color: PieceColor::White,
            })),
            'N' => board[rank].push(Some(Piece {
                kind: PieceKind::Knight,
                color: PieceColor::White,
            })),
            'B' => board[rank].push(Some(Piece {
                kind: PieceKind::Bishop,
                color: PieceColor::White,
            })),
            'Q' => board[rank].push(Some(Piece {
                kind: PieceKind::Queen,
                color: PieceColor::White,
            })),
            'K' => board[rank].push(Some(Piece {
                kind: PieceKind::King,
                color: PieceColor::White,
            })),
            'P' => board[rank].push(Some(Piece {
                kind: PieceKind::Pawn,
                color: PieceColor::White,
            })),
            _ => panic!("{} is not a valid board character in FEN", character),
        });

        assert_eq!(board.len(), 8);
        for rank in &board {
            assert_eq!(rank.len(), 8);
        }

        assert!(fields[1].len() == 1);

        let to_move = match fields[1]
            .chars()
            .nth(0)
            .expect("Second FEN field should be 'w' or 'b'")
        {
            'w' => PieceColor::White,
            'b' => PieceColor::Black,
            _ => panic!("Second FEN field should be 'w' or 'b'"),
        };

        let white_can_castle_left = fields[2].contains("Q");
        let white_can_castle_right = fields[2].contains("K");
        let black_can_castle_left = fields[2].contains("q");
        let black_can_castle_right = fields[2].contains("k");

        let en_passant_on = if fields[3] == "-" {
            None
        } else {
            Some(Coords::from_algebraic(fields[3]))
        };

        Position {
            board,
            to_move,
            en_passant_on,
            white_king_moved: false,
            black_king_moved: false,
            white_left_rook_moved: !white_can_castle_left,
            white_right_rook_moved: !white_can_castle_right,
            black_left_rook_moved: !black_can_castle_left,
            black_right_rook_moved: !black_can_castle_right,
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

        let white_left_rook_moved = match chess_move {
            ChessMove::RegularMove(movement) => {
                movement.origin == Coords { y: 0, x: 0 } || self.white_left_rook_moved
            }
            _ => self.white_left_rook_moved,
        };
        let white_right_rook_moved = match chess_move {
            ChessMove::RegularMove(movement) => {
                movement.origin == Coords { y: 0, x: 7 } || self.white_right_rook_moved
            }
            _ => self.white_right_rook_moved,
        };
        let black_left_rook_moved = match chess_move {
            ChessMove::RegularMove(movement) => {
                movement.origin == Coords { y: 7, x: 0 } || self.black_left_rook_moved
            }
            _ => self.black_left_rook_moved,
        };
        let black_right_rook_moved = match chess_move {
            ChessMove::RegularMove(movement) => {
                movement.origin == Coords { y: 7, x: 7 } || self.black_right_rook_moved
            }
            _ => self.black_right_rook_moved,
        };

        let black_king_moved = match chess_move {
            ChessMove::CastleLeft | ChessMove::CastleRight => self.to_move == PieceColor::Black,
            ChessMove::RegularMove(movement) => piece_at(&self.board, &movement.origin)
                .is_some_and(|piece| {
                    piece.kind == PieceKind::King && piece.color == PieceColor::Black
                }),
            _ => self.white_king_moved,
        };
        let white_king_moved = match chess_move {
            ChessMove::CastleLeft | ChessMove::CastleRight => self.to_move == PieceColor::White,
            ChessMove::RegularMove(movement) => piece_at(&self.board, &movement.origin)
                .is_some_and(|piece| {
                    piece.kind == PieceKind::King && piece.color == PieceColor::White
                }),
            _ => self.white_king_moved,
        };

        Position {
            board: new_board,
            to_move: self.to_move.opposite(),
            en_passant_on,
            black_king_moved,
            white_king_moved,
            black_right_rook_moved,
            black_left_rook_moved,
            white_right_rook_moved,
            white_left_rook_moved,
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
    pub fn is_move_legal(&self, chess_move: &ChessMove) -> bool {
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
    pub fn is_attacked_by(&self, by: &PieceColor, square: &Coords) -> bool {
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
        let mut moves = self.projected_movement(origin, eight_degrees(), origin_color, Some(1));
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
        self.projected_movement(origin, eight_degrees(), color, None)
    }
    fn bishop_from(&self, origin: &Coords, color: &PieceColor) -> Vec<ChessMove> {
        self.projected_movement(origin, inter_cards(), color, None)
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
        self.projected_movement(origin, cards(), color, None)
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
    fn projected_movement(
        &self,
        origin: &Coords,
        directions: Vec<Direction>,
        origin_color: &PieceColor,
        limit: Option<isize>,
    ) -> Vec<ChessMove> {
        directions
            .iter()
            .map(|dir| self.raycast(origin, dir, origin_color, limit))
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
        &self,
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
            if let Some(piece) = piece_at(&self.board, &next_square) {
                if piece.color == origin_color.opposite() {
                    squares.push(next_square);
                }
                break;
            }
            squares.push(next_square);
        }
        squares
    }
    pub fn piece_count(&self, color: PieceColor) -> usize {
        all_squares()
            .iter()
            .filter(|square| {
                piece_at(&self.board, square).is_some_and(|piece| piece.color == color)
            })
            .count()
    }

    pub fn is_stalemate(&self) -> bool {
        self.all_legal_moves().len() == 0 && !self.is_in_check(&self.to_move)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn initial_position_from_fen() {
        assert_eq!(
            Position::initial(),
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        );
    }

    #[test]
    fn finds_complex_checkmate() {
        assert!(Position::from_fen(
            "r1bqkbnr/2pp1Qpp/ppn5/4p3/2BPP3/8/PPP2PPP/RNB1K1NR b KQkq - 0 1"
        )
        .is_checkmate());
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
    fn cant_castle_after_moving_king() {
        let mut position = Position::empty_board();
        position.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[0][0] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let king_initial_location = Coords { y: 0, x: 4 };
        let one_above = Coords { y: 1, x: 4 };

        let after_move_up = position.after_move(&ChessMove::RegularMove(Move {
            origin: king_initial_location,
            destination: one_above,
        }));

        assert!(after_move_up.white_king_moved);
        assert!(!after_move_up.is_move_legal(&ChessMove::CastleLeft));
        assert!(!after_move_up.is_move_legal(&ChessMove::CastleRight));

        let after_move_back = after_move_up.after_move(&ChessMove::RegularMove(Move {
            origin: one_above,
            destination: king_initial_location,
        }));

        assert!(after_move_back.white_king_moved);
        assert!(!after_move_back.is_move_legal(&ChessMove::CastleLeft));
        assert!(!after_move_back.is_move_legal(&ChessMove::CastleRight));
    }

    #[test]
    fn cannot_castle_after_moving_rook() {
        let mut position = Position::empty_board();
        position.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        position.board[0][0] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let left_rook_initial_location = Coords { y: 0, x: 0 };
        let left_rook_up_one = Coords { y: 1, x: 0 };
        let right_rook_initial_location = Coords { y: 0, x: 7 };
        let right_rook_up_one = Coords { y: 1, x: 7 };

        let moved_left_rook_up_one = position
            .after_move(&ChessMove::RegularMove(Move {
                origin: left_rook_initial_location,
                destination: left_rook_up_one,
            }))
            .color_to_move(PieceColor::White);

        assert!(moved_left_rook_up_one.white_left_rook_moved);
        assert!(!moved_left_rook_up_one.white_right_rook_moved);

        assert!(!moved_left_rook_up_one.is_move_legal(&ChessMove::CastleLeft));
        assert!(moved_left_rook_up_one.is_move_legal(&ChessMove::CastleRight));

        let moved_right_rook_up_one = moved_left_rook_up_one
            .after_move(&ChessMove::RegularMove(Move {
                origin: right_rook_initial_location,
                destination: right_rook_up_one,
            }))
            .color_to_move(PieceColor::White);

        assert!(moved_right_rook_up_one.white_right_rook_moved);
        assert!(!moved_right_rook_up_one.is_move_legal(&ChessMove::CastleRight));

        let moved_rooks_back = moved_right_rook_up_one
            .after_move(&ChessMove::RegularMove(Move {
                origin: left_rook_up_one,
                destination: left_rook_initial_location,
            }))
            .after_move(&ChessMove::RegularMove(Move {
                origin: right_rook_up_one,
                destination: right_rook_initial_location,
            }));

        assert!(moved_rooks_back.white_right_rook_moved);
        assert!(moved_rooks_back.white_left_rook_moved);
        assert!(!moved_rooks_back.is_move_legal(&ChessMove::CastleRight));
        assert!(!moved_rooks_back.is_move_legal(&ChessMove::CastleLeft));
    }

    #[test]
    fn detects_stalemate() {
        let mut position = Position::empty_board();
        position.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        position.board[2][1] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        position.board[1][2] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });

        assert!(position.is_stalemate());
    }
}
