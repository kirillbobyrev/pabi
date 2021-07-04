include(GoogleTest)
add_executable(BoardTest board.cc)
target_link_libraries(BoardTest gtest_main gtest Threads::Threads Chess)
gtest_add_tests(TARGET BoardTest)
