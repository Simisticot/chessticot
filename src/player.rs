use crate::{ChessMove, Position};

pub trait Player {
    fn offer_move(&self, position: &Position) -> ChessMove;
}
