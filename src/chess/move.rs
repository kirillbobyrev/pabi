/// Move generation, application and parsing.

use core::{Square, PieceKind};

enum Move {
    Regular(RegularMove),
}

#[derive(Copy, Clone, Debug)]
struct RegularMove {
    from: Square,
    to: Square,
    promotion: Option<PieceKind>,
}
