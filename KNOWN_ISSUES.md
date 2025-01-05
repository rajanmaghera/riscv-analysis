# Known Issues in RISC-V Analyzer

This document lists known issues and limitations of RISC-V Analyzer.

- RISC-V floating-point instructions are not supported.
- The `.eqv` directive for symbolic constants is not supported.
- RISC-V interrupt handlers are not recognized as functions. This can cause
  spurious dead-code warnings.
