#include "board.h"

#include <iostream>

#include "notation.h"

namespace aiseu::chess {

PieceSet::PieceSet(Side owner) : owner_(owner) {
  if (owner == Side::kWhite) {
    king_position_ = AlgebraicPosition("e1");
    pawn_positions_ = {AlgebraicPosition("a2"), AlgebraicPosition("b2"),
                       AlgebraicPosition("c2"), AlgebraicPosition("d2"),
                       AlgebraicPosition("e2"), AlgebraicPosition("f2"),
                       AlgebraicPosition("g2"), AlgebraicPosition("h2)")};
  }
}

void PieceCentricBoard::Dump() const {
  std::ostream os(std::cout.rdbuf());
  Dump(os);
}

void PieceCentricBoard::DumpFigurine() const {
  std::ostream os(std::cout.rdbuf());
  Dump(os);
}

void PieceCentricBoard::DumpFEN() const {
  std::ostream os(std::cout.rdbuf());
  Dump(os);
}

}  // namespace aiseu::chess
