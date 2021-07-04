cmake_minimum_required(VERSION 3.18)

project(
  bibimbap
  VERSION 0.0.1
  DESCRIPTION "Chess engine"
  HOMEPAGE_URL https://github.com/kirillbobyrev/bibimbap
  LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)

find_package(GTest 1.10 REQUIRED)
find_package(glog 0.4 REQUIRED)
find_package(gflags 2.2 REQUIRED)
find_package(benchmark 1.5 REQUIRED)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

include_directories(${CMAKE_CURRENT_SOURCE_DIR})
add_subdirectory(bibimbap)

add_subdirectory(tests)
