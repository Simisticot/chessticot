use crate::Coords;
use crate::PieceKind;

#[derive(PartialEq, Hash, Eq, Debug, Clone)]
pub enum ChessMove {
    RegularMove(Move),
    PawnSkip(Move),
    CastleLeft,
    CastleRight,
    EnPassant(Move, Coords),
    Promotion(Move, PieceKind),
}

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub struct Move {
    pub origin: Coords,
    pub destination: Coords,
}
