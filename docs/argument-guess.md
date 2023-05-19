

# Guessing Arguments
After performing liveness analysis, we will guess which argument registers are 
in use by each function. This is for function calls and return values.

Under the RISC-V Conventions, arguments are passed in registers `a0` through `a7`.
The return value is passed in registers `a0` and `a1`. The conventions also
state how overflow arguments are passed via the stack, but we will not be
considering that for now.

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
to the list of returns for that function. If there are any return statements that
correspond to more than or less than 1 function, we raise a warning and cease
guessing arguments for that statement **TODO**.

## First Guess for Arguments
Once we have a list of functions and returns, we do a simple guess. At the entry
of each function, we check the OUT set before the first instruction. Masking the
OUT set with the argument registers, we get a set of registers that are used
as arguments.

Any other registers that are present in the OUT set are considered to be
invalid arguments. A separate pass will be made to check for invalid arguments
and raise warnings **TODO**.

These arguments are then added to the use set of all function calls that 
correspond to that function. This is done for all functions.

## First Guess for Return Values
At the call site of each function, we check the OUT set after the call instruction.
Masking the OUT set with the return value registers, we get a set of registers
that are (likely) used as return values. These return values are then added to
the USE set of the return statement that corresponds to that function.