/// Arrays and values generated at or before build time.
use crate::chess::bitboard::Bitboard;
use crate::chess::core::{Piece, Square, BOARD_SIZE};
use crate::chess::zobrist::Key;

// All keys required for Zobrist hashing of a chess position.
pub(super) const BLACK_TO_MOVE: Key = 0x9E06BAD39D761293;

pub(super) const WHITE_CAN_CASTLE_SHORT: Key = 0xF05AC573DD61D323;
pub(super) const WHITE_CAN_CASTLE_LONG: Key = 0x41D8B55BA5FEB78B;

pub(super) const BLACK_CAN_CASTLE_SHORT: Key = 0x680988787A43D289;
pub(super) const BLACK_CAN_CASTLE_LONG: Key = 0x2F941F8DFD3E3D1F;

pub(super) const EN_PASSANT_FILES: [Key; 8] =
    include!(concat!(env!("OUT_DIR"), "/en_passant_zobrist_keys"));

const PIECES_ZOBRIST_KEYS: [Key; 768] = include!(concat!(env!("OUT_DIR"), "/pieces_zobrist_keys"));

pub(super) fn get_piece_key(piece: Piece, square: Square) -> Key {
    const NUM_PIECES: usize = 6;
    PIECES_ZOBRIST_KEYS[piece.owner as usize * NUM_PIECES * BOARD_SIZE as usize
        + piece.kind as usize * BOARD_SIZE as usize
        + square as usize]
}

const BISHOP_ATTACKS_COUNT: usize = 5248;
pub(super) const BISHOP_ATTACKS: [Bitboard; BISHOP_ATTACKS_COUNT] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/bishop_attacks.rs"
));
pub(super) const BISHOP_ATTACK_OFFSETS: [usize; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/bishop_attack_offsets.rs"
));
pub(super) const BISHOP_RELEVANT_OCCUPANCIES: [u64; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/bishop_relevant_occupancies.rs"
));

const ROOK_ATTACKS_COUNT: usize = 102_400;
pub(super) const ROOK_ATTACKS: [Bitboard; ROOK_ATTACKS_COUNT] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/rook_attacks.rs"
));
pub(super) const ROOK_RELEVANT_OCCUPANCIES: [u64; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/rook_relevant_occupancies.rs"
));
pub(super) const ROOK_ATTACK_OFFSETS: [usize; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/rook_attack_offsets.rs"
));

pub(super) const RAYS: [Bitboard; BOARD_SIZE as usize * BOARD_SIZE as usize] =
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/rays.rs"));
pub(super) const BISHOP_RAYS: [Bitboard; BOARD_SIZE as usize * BOARD_SIZE as usize] = include!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/generated/bishop_rays.rs")
);
pub(super) const ROOK_RAYS: [Bitboard; BOARD_SIZE as usize * BOARD_SIZE as usize] = include!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/generated/rook_rays.rs")
);

pub(super) const KNIGHT_ATTACKS: [Bitboard; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/knight_attacks.rs"
));
pub(super) const KING_ATTACKS: [Bitboard; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/king_attacks.rs"
));
pub(super) const WHITE_PAWN_ATTACKS: [Bitboard; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/white_pawn_attacks.rs"
));
pub(super) const BLACK_PAWN_ATTACKS: [Bitboard; BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/black_pawn_attacks.rs"
));
