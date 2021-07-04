#include "bibimbap/chess/board.h"
#include "gtest/gtest.h"

namespace bibimbap::chess {

TEST(Board, Printer) {
  PieceCentricBoard board;
  board.Dump();
}

}  // namespace bibimbap::chess
