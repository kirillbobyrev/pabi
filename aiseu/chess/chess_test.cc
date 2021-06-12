#include "board.h"
#include "gtest/gtest.h"

namespace aiseu::chess {

TEST(Board, Printer) {
  PieceCentricBoard board;
  board.Dump();
}

}  // namespace aiseu::chess
