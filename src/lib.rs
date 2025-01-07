use core::panic;
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

    pub fn pawn_from(&self, origin: &Coords, color: &PieceColor) -> Vec<Move> {
        let direction = match color {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };

        let mut legal_moves = vec![];
        let ahead_one = Coords {
            x: origin.x,
            y: origin.y + direction,
        };
        if self.piece_at(&ahead_one).is_none() {
            legal_moves.push(Move {
                origin: origin.clone(),
                destination: ahead_one,
            });
            if origin.y == 1 || origin.y == 6 {
                legal_moves.push(Move {
                    origin: origin.clone(),
                    destination: Coords {
                        x: origin.x,
                        y: origin.y + (2 * direction),
                    },
                });
            }
        }

        vec![
            Coords {
                x: origin.x + 1,
                y: origin.y + direction,
            },
            Coords {
                x: origin.x - 1,
                y: origin.y + direction,
            },
        ]
        .iter()
        .for_each(|diagonal| match self.piece_at(&diagonal) {
            None => {}
            Some(piece) => {
                if piece.color == color.opposite() {
                    legal_moves.push(Move {
                        origin: origin.clone(),
                        destination: *diagonal,
                    });
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

#[derive(PartialEq, Debug)]
pub struct Move {
    pub origin: Coords,
    pub destination: Coords,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor,
}

impl Piece {
    fn from_initial_position(x: isize, y: isize) -> Option<Piece> {
        let color = match y {
            0 | 1 => Some(PieceColor::White),
            6 | 7 => Some(PieceColor::Black),
            _ => None,
        };
        let kind = match y {
            1 | 6 => Some(PieceKind::Pawn),
            0 | 7 => match x {
                0 | 7 => Some(PieceKind::Rook),
                1 | 6 => Some(PieceKind::Knight),
                2 | 5 => Some(PieceKind::Bishop),
                3 => Some(PieceKind::Queen),
                4 => Some(PieceKind::King),
                _ => panic!("Row should not be over 8 squares."),
            },
            _ => None,
        };
        if kind.is_none() || color.is_none() {
            None
        } else {
            Some(Piece {
                kind: kind.unwrap(),
                color: color.unwrap(),
            })
        }
    }
}

#[derive(Copy, Clone)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
