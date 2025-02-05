use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub fn promoteable() -> std::slice::Iter<'static, PieceKind> {
        [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
        ]
        .iter()
    }
}

#[derive(Eq, Hash, Copy, Clone, PartialEq, Debug)]
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
    pub fn homerow(&self) -> isize {
        match self {
            PieceColor::White => 0,
            PieceColor::Black => 7,
        }
    }
    pub fn pawn_orientation(&self) -> isize {
        match self {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        }
    }

    pub fn both() -> std::array::IntoIter<PieceColor, 2> {
        [PieceColor::White, PieceColor::Black].into_iter()
    }
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::Black => write!(f, "Black"),
            PieceColor::White => write!(f, "White"),
        }
    }
}
