mod coords;
mod piece;

pub use crate::coords::{Coords, Direction, Move};
pub use crate::piece::{Piece, PieceColor, PieceKind};
use std::usize;

pub struct Game {
    pub board: Vec<Vec<Option<Piece>>>,
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
        Game { board }
    }

    pub fn empty() -> Game {
        let mut board = Vec::new();
        for _ in 0..8 {
            let mut row = Vec::new();
            for _ in 0..8 {
                row.push(None);
            }
            board.push(row);
        }
        Game { board }
    }

    pub fn is_move_legal(&self, chess_move: &Move) -> bool {
        self.legal_moves_from_origin(&chess_move.origin)
            .contains(chess_move)
    }
    pub fn legal_moves_from_origin(&self, origin: &Coords) -> Vec<Move> {
        match self.piece_at(origin) {
            None => Vec::new(),
            Some(piece) => match piece.kind {
                PieceKind::Pawn => self.pawn_from(origin, &piece.color),
                PieceKind::Rook => self.rook_from(origin, &piece.color),
                PieceKind::Knight => self.knight_from(origin, &piece.color),
                _ => all_squares()
                    .iter()
                    .map(|square| Move {
                        origin: origin.clone(),
                        destination: *square,
                    })
                    .collect(),
            },
        }
    }

    fn knight_from(&self, origin: &Coords, color: &PieceColor) -> Vec<Move> {
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
        let potential_moves = directions.iter().map(|direction| Move {
            origin: origin.clone(),
            destination: *origin + *direction,
        });
        potential_moves
            .into_iter()
            .filter(|chess_move| {
                chess_move.destination.is_in_bounds()
                    && self
                        .piece_at(&chess_move.destination)
                        .is_none_or(|piece| &piece.color != color)
            })
            .collect()
    }

    fn rook_from(&self, origin: &Coords, color: &PieceColor) -> Vec<Move> {
        let mut legal_moves: Vec<Move> = Vec::new();
        let up = Direction { dx: 0, dy: 1 };
        let down = Direction { dx: 0, dy: -1 };
        let left = Direction { dx: -1, dy: 0 };
        let right = Direction { dx: 1, dy: 0 };
        let sides = vec![up, down, left, right];
        sides.iter().for_each(|direction| {
            let mut moves: Vec<Move> = self
                .raycast(origin, direction, color)
                .iter()
                .map(|destination| Move {
                    origin: origin.clone(),
                    destination: *destination,
                })
                .collect();
            legal_moves.append(&mut moves);
        });
        legal_moves
    }

    fn pawn_from(&self, origin: &Coords, color: &PieceColor) -> Vec<Move> {
        let vertical_orientation = match color {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };
        let mut legal_moves = vec![];
        let forward = Direction {
            dx: 0,
            dy: vertical_orientation,
        };
        let ahead_one = *origin + forward;
        let ahead_two = ahead_one + forward;

        if !ahead_one.is_in_bounds() {
            return legal_moves;
        }

        if self.piece_at(&ahead_one).is_none() {
            legal_moves.push(Move {
                origin: origin.clone(),
                destination: ahead_one,
            });
            if (origin.y == 1 || origin.y == 6) && self.piece_at(&ahead_two).is_none() {
                legal_moves.push(Move {
                    origin: origin.clone(),
                    destination: ahead_two,
                });
            }
        }

        vec![
            Coords {
                x: origin.x + 1,
                y: origin.y + vertical_orientation,
            },
            Coords {
                x: origin.x - 1,
                y: origin.y + vertical_orientation,
            },
        ]
        .iter()
        .for_each(|diagonal| {
            if diagonal.is_in_bounds() {
                match self.piece_at(&diagonal) {
                    None => {}
                    Some(piece) => {
                        if piece.color == color.opposite() {
                            legal_moves.push(Move {
                                origin: origin.clone(),
                                destination: *diagonal,
                            });
                        }
                    }
                }
            }
        });
        legal_moves
    }

    pub fn make_move(&mut self, chess_move: &Move) {
        if self.is_move_legal(chess_move) {
            self.move_piece(chess_move.origin, chess_move.destination);
        }
    }

    pub fn move_piece(&mut self, origin: Coords, dest: Coords) {
        if let Some(origin_piece) = self.take_piece_at(origin) {
            self.put_piece_at(origin_piece, dest);
        }
    }
    pub fn piece_at(&self, loc: &Coords) -> Option<Piece> {
        self.board[loc.y as usize][loc.x as usize].clone()
    }
    pub fn take_piece_at(&mut self, loc: Coords) -> Option<Piece> {
        self.board[loc.y as usize][loc.x as usize].take()
    }
    pub fn put_piece_at(&mut self, piece: Piece, loc: Coords) {
        self.board[loc.y as usize][loc.x as usize] = Some(piece);
    }
    pub fn raycast(
        &self,
        origin: &Coords,
        direction: &Direction,
        origin_color: &PieceColor,
    ) -> Vec<Coords> {
        let mut squares = vec![];
        // for instead of loop to avoid potential infinite loop
        for i in 1..8 {
            let next_square = *origin + (*direction * i);
            if !next_square.is_in_bounds() {
                break;
            }
            if let Some(piece) = self.piece_at(&next_square) {
                if piece.color == origin_color.opposite() {
                    squares.push(next_square);
                }
                break;
            }
            squares.push(next_square);
        }
        squares
    }
}

fn all_squares() -> Vec<Coords> {
    let mut squares = Vec::new();
    for i in 0..8 {
        for j in 0..8 {
            squares.push(Coords { x: j, y: i });
        }
    }
    squares
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash::RandomState};

    use super::*;

    #[test]
    fn pawn_homerow() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 2, x: 4 }
                },
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                }
            ]
        )
    }

    #[test]
    fn pawn_not_homerow() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![Move {
                origin: pawn_location,
                destination: Coords { y: 3, x: 4 }
            }]
        )
    }

    #[test]
    fn pawn_not_homerow_with_capture() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                },
                Move {
                    origin: pawn_location,
                    destination: opposing_pawn_location
                }
            ]
        )
    }

    #[test]
    fn pawn_not_homerow_blocked() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(game.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_edge_of_board_horizontal_blocked() {
        let mut game = Game::empty();
        game.board[2][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 7 };
        assert_eq!(game.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_edge_of_board_vertical() {
        let mut game = Game::empty();
        game.board[7][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 7, x: 4 };
        assert_eq!(game.legal_moves_from_origin(&pawn_location), vec![])
    }

    #[test]
    fn pawn_not_homerow_with_capture_blocked() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![Move {
                origin: pawn_location,
                destination: opposing_pawn_location
            }]
        )
    }

    #[test]
    fn pawn_homerow_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(game.legal_moves_from_origin(&pawn_location), vec![])
    }
    #[test]
    fn pawn_homerow_second_square_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![Move {
                origin: pawn_location,
                destination: Coords { y: 2, x: 4 }
            }]
        )
    }

    #[test]
    fn pawn_homerow_with_capture_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.board[2][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        let capture_location = Coords { y: 2, x: 5 };
        assert_eq!(
            game.legal_moves_from_origin(&pawn_location),
            vec![Move {
                origin: pawn_location,
                destination: capture_location
            }]
        )
    }

    #[test]
    fn rook_middle_board() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let mut legal_moves = vec![];

        for j in 0..8 {
            if j != 4 {
                legal_moves.push(Move {
                    origin: rook_location,
                    destination: Coords { y: 4, x: j },
                });
            }
        }
        for i in 0..8 {
            if i != 4 {
                legal_moves.push(Move {
                    origin: rook_location,
                    destination: Coords { x: 4, y: i },
                });
            }
        }

        assert_eq!(
            legal_moves.len(),
            game.legal_moves_from_origin(&rook_location).len()
        );
        legal_moves.iter().for_each(|chess_move| {
            assert!(game
                .legal_moves_from_origin(&rook_location)
                .contains(chess_move))
        });
    }

    #[test]
    fn rook_middle_board_boxed_in_opposite_color() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let up = Coords { y: 5, x: 4 };
        let down = Coords { y: 3, x: 4 };
        let left = Coords { y: 4, x: 3 };
        let right = Coords { y: 4, x: 5 };

        let legal_moves = vec![
            Move {
                origin: rook_location,
                destination: up,
            },
            Move {
                origin: rook_location,
                destination: down,
            },
            Move {
                origin: rook_location,
                destination: left,
            },
            Move {
                origin: rook_location,
                destination: right,
            },
        ];

        assert_eq!(game.legal_moves_from_origin(&rook_location), legal_moves);
    }

    #[test]
    fn rook_middle_board_boxed_in_own_color() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };

        let legal_moves = vec![];

        assert_eq!(game.legal_moves_from_origin(&rook_location), legal_moves);
    }

    #[test]
    fn knight_middle_board() {
        let mut game = Game::empty();
        game.board[3][3] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 3, x: 3 };

        let legal_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            vec![
                Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 4 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 5 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 2 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 1 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 4 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 5 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                },
            ]
            .iter()
            .cloned(),
        );

        assert_eq!(
            legal_moves,
            HashSet::from_iter(
                game.legal_moves_from_origin(&knight_location)
                    .iter()
                    .cloned()
            )
        );
    }

    #[test]
    fn knight_corner() {
        let mut game = Game::empty();
        game.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        let legal_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            vec![
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                },
            ]
            .iter()
            .cloned(),
        );

        assert_eq!(
            legal_moves,
            HashSet::from_iter(
                game.legal_moves_from_origin(&knight_location)
                    .iter()
                    .cloned()
            )
        )
    }

    #[test]
    fn knight_corner_blocked() {
        let mut game = Game::empty();
        game.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        game.board[1][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        game.board[2][1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        assert_eq!(game.legal_moves_from_origin(&knight_location).len(), 0)
    }
}
