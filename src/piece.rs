#[derive(Copy, Clone)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor,
}

impl Piece {
    pub fn from_initial_position(x: isize, y: isize) -> Option<Piece> {
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

#[derive(Copy, Clone, PartialEq, Debug)]
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
