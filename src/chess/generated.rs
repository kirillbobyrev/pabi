use crate::chess::bitboard::Bitboard;
use crate::chess::zobrist::Key;
use crate::chess::core::BOARD_SIZE;

const BLACK_TO_MOVE: Key = 0;

const WHITE_CAN_CASTLE_KINGSIDE: Key = 1;
const BLACK_CAN_CASTLE_KINGSIDE: Key = 1;
const WHITE_CAN_CASTLE_QUEENSIDE: Key = 1;
const BLACK_CAN_CASTLE_QUEENSIDE: Key = 1;

const EN_PASSANT_FILES: [Key; 8] = include!(concat!(env!("OUT_DIR"), "/en_passant_zobrist_keys"));

const WHITE_KING: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const WHITE_QUEEN: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const WHITE_ROOK: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const WHITE_BISHOP: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const WHITE_KNIGHT: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const WHITE_PAWN: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));

const BLACK_KING: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const BLACK_QUEEN: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const BLACK_ROOK: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const BLACK_BISHOP: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const BLACK_KNIGHT: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
const BLACK_PAWN: [Key; 64] = include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));

// Generated in build.rs.
// TODO: Document PEXT bitboards.
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
pub(super) const BISHOP_RAYS: [Bitboard; BOARD_SIZE as usize * BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/bishop_rays.rs"
));
pub(super) const ROOK_RAYS: [Bitboard; BOARD_SIZE as usize * BOARD_SIZE as usize] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/rook_rays.rs"
));

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
