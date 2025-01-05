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
    pub fn legal_destinations_from_origin(&self, origin: &Coords) -> Vec<Coords> {
        match self.piece_at(origin) {
            None => Vec::new(),
            Some(piece) => match piece.kind {
                PieceKind::Pawn => match piece.color {
                    PieceColor::White => {
                        let mut destinations = vec![Coords {
                            x: origin.x,
                            y: origin.y + 1,
                        }];
                        if origin.y == 1 || origin.y == 6 {
                            destinations.push(Coords {
                                x: origin.x,
                                y: origin.y + 2,
                            });
                        }
                        destinations
                    }
                    PieceColor::Black => {
                        let mut destinations = vec![Coords {
                            x: origin.x,
                            y: origin.y - 1,
                        }];
                        if origin.y == 1 || origin.y == 6 {
                            destinations.push(Coords {
                                x: origin.x,
                                y: origin.y - 2,
                            });
                        }
                        destinations
                    }
                },
                _ => all_squares(),
            },
        }
    }

    pub fn make_move(&mut self, chess_move: Move) {
        self.move_piece(chess_move.origin, chess_move.destination);
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

pub struct Move {
    pub origin: Coords,
    pub destination: Coords,
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub enum PieceColor {
    Black,
    White,
}

pub fn piece_display_name(kind: &PieceKind) -> String {
    match kind {
        PieceKind::Pawn => "Pawn".to_string(),
        PieceKind::Rook => "Rook".to_string(),
        PieceKind::Knight => "Knight".to_string(),
        PieceKind::Bishop => "Bishob".to_string(),
        PieceKind::Queen => "Queen".to_string(),
        PieceKind::King => "King".to_string(),
    }
}
