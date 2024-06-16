use crate::chess::transposition::Key;

pub(super) const BLACK_TO_MOVE: Key = 0x9e06bad39d761293;

pub(super) const WHITE_CAN_CASTLE_KINGSIDE: Key = 0xf05ac573dd61d323;
pub(super) const BLACK_CAN_CASTLE_KINGSIDE: Key = 0x680988787a43d289;
pub(super) const WHITE_CAN_CASTLE_QUEENSIDE: Key = 0x41d8b55ba5feb78b;
pub(super) const BLACK_CAN_CASTLE_QUEENSIDE: Key = 0x2f941f8dfd3e3d1f;

pub(super) const EN_PASSANT_FILES: [Key; 8] =
    include!(concat!(env!("OUT_DIR"), "/en_passant_zobrist_keys"));

pub(super) const WHITE_KING: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const WHITE_QUEEN: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const WHITE_ROOK: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const WHITE_BISHOP: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const WHITE_KNIGHT: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const WHITE_PAWN: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));

pub(super) const BLACK_KING: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const BLACK_QUEEN: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const BLACK_ROOK: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const BLACK_BISHOP: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const BLACK_KNIGHT: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
pub(super) const BLACK_PAWN: [Key; 64] =
    include!(concat!(env!("OUT_DIR"), "/white_king_zobrist_keys"));
