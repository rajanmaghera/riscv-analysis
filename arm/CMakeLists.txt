cmake_minimum_required(VERSION 3.0)
project(ArmParser)

# Get LLVM
include("${CMAKE_SOURCE_DIR}/cmake/get_llvm.cmake")

# Set C++ standards.
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall")

# Include project files
include_directories("${CMAKE_SOURCE_DIR}/include")
add_subdirectory("${CMAKE_SOURCE_DIR}/src")
