use std::fmt;

const BOARD_LENGTH: usize = 8;
const SQUARES_COUNT: usize = BOARD_LENGTH * BOARD_LENGTH;

struct MailboxBoard {
    // (Possibly) occupied squares.
    occupied_squares: [Option<Piece>; SQUARES_COUNT],
}

impl MailboxBoard {
    // TODO: Use a constant evaluation/constant board for initialization for speed and simplicity.
    #[rustfmt::skip]
    fn new() -> Self {
        MailboxBoard {
            occupied_squares: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
        }
    }

    fn put_piece(&mut self, square: Square, owner: Player, kind: PieceKind) {
        debug_assert!(
            self.occupied_squares[square as usize].is_none(),
            "Can't put piece to already occupied square"
        );
        self.occupied_squares[square as usize] = Some(Piece { owner, kind });
    }

    fn clear_square(&mut self, square: Square) {
        debug_assert!(
            self.occupied_squares[square as usize].is_some(),
            "Can't clear square that is already empty"
        );
        self.occupied_squares[square as usize] = None;
    }
}

#[derive(Copy, Clone)]
struct Piece {
    owner: Player,
    kind: PieceKind,
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[rustfmt::skip]
enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8, // Rank 8
    A7, B7, C7, D7, E7, F7, G7, H7, // Rank 7
    A6, B6, C6, D6, E6, F6, G6, H6, // Rank 6
    A5, B5, C5, D5, E5, F5, G5, H5, // Rank 5
    A4, B4, C4, D4, E4, F4, G4, H4, // Rank 4
    A3, B3, C3, D3, E3, F3, G3, H3, // Rank 3
    A2, B2, C2, D2, E2, F2, G2, H2, // Rank 2
    A1, B1, C1, D1, E1, F1, G1, H1, // Rank 1
}

#[derive(Copy, Clone, Debug)]
enum Player {
    White,
    Black,
}

#[derive(Copy, Clone, Debug)]
enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    fn relative_value(&self) -> Option<u32> {
        match &self {
            // The value of King is undefined as it cannot be captured.
            PieceKind::King => None,
            PieceKind::Queen => Some(9),
            PieceKind::Rook => Some(6),
            PieceKind::Bishop => Some(3),
            PieceKind::Knight => Some(3),
            PieceKind::Pawn => Some(1),
        }
    }
}

impl fmt::Display for PieceKind {
    // TODO: Displaying the figurine symbols ('♔', '♛', '♖' etc) requires knowing its owner.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let algebraic_symbol = match &self {
            PieceKind::King => 'K',
            PieceKind::Queen => 'Q',
            PieceKind::Rook => 'R',
            PieceKind::Bishop => 'B',
            PieceKind::Knight => 'K',
            // Pawn does not have algebraic symbol but we'll use 'p' for debugging.
            PieceKind::Pawn => 'p',
        };
        write!(f, "{}", algebraic_symbol)
    }
}

#[cfg(test)]
mod test {
    use super::MailboxBoard;

    fn new_board() {
        let board = MailboxBoard::new();
    }
}
