# Gather source files
set(
  arm_parser_src_files
    "${CMAKE_CURRENT_SOURCE_DIR}/main.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/streamer.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/instruction.cc"
)

# Build the executable
add_executable(arm-parser ${arm_parser_src_files})

# Link with LLVM
target_link_libraries(arm-parser LLVM)
