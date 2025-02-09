find_package(LLVM REQUIRED CONFIG)

# Status messages about LLVM found.
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using Config.cmake in: ${LLVM_DIR}")

message("LLVM STATUS:
  Definitions ${LLVM_DEFINITIONS}
  Includes    ${LLVM_INCLUDE_DIRS}
  Libraries   ${LLVM_LIBRARY_DIRS}
  Targets     ${LLVM_TARGETS_TO_BUILD}"
)

# Add LLVM directories
include_directories(SYSTEM "${LLVM_INCLUDE_DIRS}")
link_directories(${LLVM_LIBRARY_DIRS})
add_definitions(${LLVM_DEFINITIONS})
