

# Guessing Arguments
After performing liveness analysis, we will guess which argument registers are 
in use by each function. This is for function calls and return values.

Under the RISC-V Conventions, arguments are passed in registers `a0` through `a7`.
The return value is passed in registers `a0` and `a1`. The conventions also
state how overflow arguments are passed via the stack, but we will not be
considering that for now.

## Defining Arguments and Return Values
Because argument registers are caller-saved, some values might be used as both
arguments and return values. Some might also be never used.

Unless the arguments are explicitly defined by a programmer, arguments are guessed
by the compiler. This represents a subset of allowed programs, but this subset
**(should be/)**is guaranteed to be a safe set of arguments. We will be guessing
arguments based on the following rules:
- Return values are used by the caller after a function call. Unused values are
  not considered to be return values.
- A return value must be defined within a function. Arguments passed to a function
  not defined within a function
- Nested calls act as bars, and therefore only values after the last call is
  considered to be a return value.
- A return value must be set in all branches of a function. If a return value
  is not set in a branch, it is not considered to be a return value.
  (Note: if a potential return value is conditionally set, it is considered an
  error).

## Function Calls
A function is denoted by a jump and link (jal) call to a label. This call
sets the return address register (ra) to the address of the next instruction
and jumps to the label. The label is the name of the function. We examine the
code for any jal instructions and add the function name to a list of functions.
If there are any function labels that also have a conditional or unconditional
jump where the return address is not set conventionally, we raise a warning **TODO**.

## Return Statements
A return statement is denoted by a jump and link register (jalr) call to the
return address register (ra). This call jumps to the address stored in ra. In order
to estimate arguments correctly, we must know which function is being returned from.
To do this, we enforce that each return statement must correspond to one function
only. We examine the code for any return statements. We can then walk backwards
from the return statement to find a function label. We then add the return statement
to the list of returns for that function. If a return statement does not correspond to
a function, or a return statement is found to link to more than 1 label, an error is raised

**TODO** All return statements are transformed into conditional jumps to a new basic block
representing an **EXIT** node. This is done to allow multiple return statements in a function
while preserving conventions for dataflow analysis.

**TODO** Insert LaTeX algorithms
