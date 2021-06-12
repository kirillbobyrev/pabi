#include "aiseu/chess/notation.h"

#include <cassert>

#include "absl/base/macros.h"
#include "aiseu/chess/board.h"

namespace aiseu::chess {

namespace {

// To disambiguate between pieces of the opposite sides in FEN uppercase letters
// are used for white pieces and lowercase for black.
const std::unordered_map<PieceKind, char> kWhitePieceToAlgebraic = {
    {PieceKind::kKing, 'K'},   {PieceKind::kQueen, 'Q'},
    {PieceKind::kRook, 'R'},   {PieceKind::kBishop, 'B'},
    {PieceKind::kKnight, 'K'}, {PieceKind::kPawn, 'P'},
};
const std::unordered_map<PieceKind, char> kBlackPieceToAlgebraic = {
    {PieceKind::kKing, 'k'},   {PieceKind::kQueen, 'q'},
    {PieceKind::kRook, 'r'},   {PieceKind::kBishop, 'b'},
    {PieceKind::kKnight, 'k'}, {PieceKind::kPawn, 'p'},
};

// Figurine symbols for black and white pieces are different. Example: ♔ and ♚,
// ♖ and ♜.
const std::unordered_map<PieceKind, char> kWhitePieceToFigurine = {
    {PieceKind::kKing, u'\u2654'},   {PieceKind::kQueen, u'\u2655'},
    {PieceKind::kRook, u'\u2656'},   {PieceKind::kBishop, u'\u2657'},
    {PieceKind::kKnight, u'\u2658'}, {PieceKind::kPawn, u'\u2659'},
};
const std::unordered_map<PieceKind, char> kBlackPieceToFigurine = {
    {PieceKind::kKing, u'\u2654'},   {PieceKind::kQueen, u'\u2655'},
    {PieceKind::kRook, u'\u2656'},   {PieceKind::kBishop, u'\u2657'},
    {PieceKind::kKnight, u'\u2658'}, {PieceKind::kPawn, u'\u2659'},
};

}  // namespace

// TODO(kirillbobyrev): Make these two functions constexpr for better
// performance? Likely to result in k(Color)PieceTo(Algebraic|Figurine) being
// exposed in the header.
std::uint8_t FileToNumeric(char file) {
  ABSL_ASSERT('a' <= file && file <= 'h');
  return static_cast<std::int8_t>(file - 'a');
}

Position AlgebraicPosition(std::string_view position) {
  ABSL_ASSERT(position.size() == 2);
  ABSL_ASSERT('1' <= position[1] && position[1] <= '8');
  return Position{.file = FileToNumeric(position[0]),
                  .rank = static_cast<std::uint8_t>(position[1] - '0')};
}

void PieceCentricBoard::Dump(std::ostream &os) const {}

}  // namespace aiseu::chess
