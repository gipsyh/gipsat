cmake_minimum_required(VERSION 3.5)
project(GipSAT)
set(CMAKE_CXX_STANDARD 11)

include_directories(${CMAKE_CURRENT_SOURCE_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../transys/cpp)
add_library(GipSAT STATIC gipsat.c)

add_custom_target(
    libgipsat
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/..
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/../src/*.rs
)
add_dependencies(GipSAT libgipsat)

target_include_directories(GipSAT PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(GipSAT ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libgipsat.a)
