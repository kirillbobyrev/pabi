#ifndef BIBIMBAP_CHESS_BOARD_H_
#define BIBIMBAP_CHESS_BOARD_H_

#include <array>
#include <cstdint>
#include <iterator>
#include <ostream>
#include <string_view>

namespace bibimbap::chess {

enum class Side : std::uint8_t {
  kWhite = 0,
  kBlack,
};

enum class PieceKind : std::uint8_t {
  kKing = 0,
  kQueen,
  kRook,
  kBishop,
  kKnight,
  kPawn,
};

// TODO(kirillbobyrev): There are only 64 positions, would most likely be faster
// to use a single uint8_t for Position. The best way to figure it out would be
// through a set of benchmarks.
struct Position {
  std::uint8_t file;
  std::uint8_t rank;
};

class Board {
 public:
  // Draws the board and pieces in the algebraic format (KQRBNKP for white and
  // kqrbnkp for black).
  // NOTE: Implementations are located in bibimbap/chess/notation.cc.
  virtual void Dump(std::ostream &os) const = 0;
  virtual void Dump() const = 0;
  // Draws the board with pieces in the figurine format (uses Unicode piece
  // symbols such as ♖ and ♜).
  virtual void DumpFigurine(std::ostream &os) const = 0;
  virtual void DumpFigurine() const = 0;
  // Prints the board using Forsyth–Edwards Notation (FEN).
  //
  // This is not the full FEN since it does not contain any information about
  // the current turn, castling and en passant availability. The full FEN can be
  // dumped from the Game instance.
  virtual void DumpFEN(std::ostream &os) const = 0;
  virtual void DumpFEN() const = 0;
  // TODO(kirillbobyrev): Board interface should be generic enough to avoid
  // derived class-specific dump implementations.
};

class PieceSet {
 public:
  // Creates the piece set for given player at the start of the game.
  explicit PieceSet(Side owner);

 private:
  const Side owner_;

  static constexpr std::int8_t kInitialPawnsCount = 8;
  static constexpr std::int8_t kInitialKnightsCount = 8;
  static constexpr std::int8_t kInitialBishopsCount = 8;
  static constexpr std::int8_t kInitialRooksCount = 8;

  std::int8_t num_pawns_ = kInitialPawnsCount;
  std::int8_t num_knights_ = kInitialPawnsCount;
  std::int8_t num_bishops_ = kInitialPawnsCount;
  std::int8_t num_rooks_ = kInitialPawnsCount;
  bool has_queen_ = true;

  Position king_position_;
  Position queen_position_;
  std::array<Position, kInitialPawnsCount> pawn_positions_;
  std::array<Position, kInitialKnightsCount> knight_positions_;
  std::array<Position, kInitialBishopsCount> bishop_positions_;
  std::array<Position, kInitialRooksCount> rook_positions_;

  friend class PieceCentricBoard;
};

class PieceCentricBoard : public Board {
 public:
  // Creates the board in the beginning of the game.
  PieceCentricBoard() = default;
  // Builds a board given FEN description. Advances iterator to the end of FEN
  // board description.
  explicit PieceCentricBoard(std::istream_iterator<char> *fen_state);

  void Dump(std::ostream &os) const override;
  void Dump() const override;

  void DumpFigurine(std::ostream &os) const override;
  void DumpFigurine() const override;

  void DumpFEN(std::ostream &os) const override;
  void DumpFEN() const override;

 private:
  PieceSet black_pieces_ = PieceSet(Side::kBlack);
  PieceSet white_pieces_ = PieceSet(Side::kBlack);
};

}  // namespace bibimbap::chess

#endif  // BIBIMBAP_CHESS_BOARD_H_
