Concepts
-   Only the strong survive
-   Ownership
-   Struggle and death
-   Sacrifice
-   Starvation
-   Layer hierarchy
-   Suicide

In a MindBend program, there are only 15 memory locations called cells
There are primitives which can only be gotten after drilling through the layers:
    input, output, numbers 0 through 9, all in their own specially named locations
    beneath the layers

Everything is an expression

All expressions are living things
They live from the moment they are created
If an expression isn't leached onto a memory cell, it dies immediately
An expression must be leached onto a memory cell to survive
After the creation of 3 expressions, the least accessed expression experiences starvation due to the limited
    resources in the data landscape and commits suicide. The death expression will occupy its cell
It's up to you, the MindBendee, to keep track of the available memory cells and use them efficiently


There is only addition and subtraction, no multiplication or division

There are 5 types of expressions:
    organism expressions - the expression of expressions which makes up a MindBend program
    drill expressions - an expression to get lower in the layer hierarchy to get to a primitive
    leach expression - an expression to associate a host expression with a memory cell
    death expression - an expression to disassociate an expression from a cell and leave it to die

At the end of the program's execution, the organism expression, which is the program itself, dies


The core MindBend concepts:
-   Expressions: living things which express the snapshot of a computation in an organism
-   Cell: A memory location which an expression ought to be leached onto to survive
-   Layers: a memory location protected by 3 gates which must be drilled though to access the primitives
-   Primitives: eternal expressions that live beneath the layers
-   Suicide: the self killing of the least accessed expression in the Cells after the creation of 5 expressions 
    and the subsequent filling of the cell with the death expression
-   Gate: a cover which must be drilled though to access what lies beyond it
-   Self-Close: the automatic closing of the gates after 5 expressions have been created
-   Organism: the expression of expressions, the MindBend program itself, a cluster of expressions
    representing a unit of computation


Primitives
These include the numbers 0 to 9, the input, output functions, addition and subtraction expressions
The primitive operator, $, followed by a primitive symbol or index is used to
access a primitive.
Primitives can only be accessed after the 3 gates through the layers have been opened
After the creation of 5 expressions, the gates close
The primitives can also be identified by their primitive index

The Primitive Table (primitive symbol -> primitive index -> primitive result)
!   ->  D   ->  1
@   ->  C   ->  2
#   ->  B   ->  3
+   ->  A   ->  4
%   ->  9   ->  5
`   ->  8   ->  6
&   ->  7   ->  7
*   ->  6   ->  8
(   ->  5   ->  9
)   ->  4   ->  0
<>  ->  3   ->  input
><  ->  2   ->  output
}   ->  1   ->  addition
{   ->  0   ->  subtraction
\\|//\\|//\\|//$(~0$&~1$*~2$#~4$+~5//|\\\\|//\\|//^^^^^^666^^^^^^=00~0
$(~$)~$&~$*~$%~$+~$!

After all 3 gates have been opened, 5 more expressions, before they close.
An expression dies 5 expressions after it is leached onto a Cell

->L\\|//\\|//\\|//$`->C~0->L$><->C~10~1
->L\\|//\\|//\\|//$%->C~0->L$<>->C~1
Print the letter 'A'
->L\\|//\\|//\\|//$`->C~0->L$%->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~97~A8~B9~B~A^^^^^^666^^^^^^=M^^^^^^666^^^^^^=O

Print the numbers 1 to 5 without looping
->L\\|//\\|//\\|//$+->C~0->L$(->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M->L\\|//\\|//\\|//$%->C~0->L$)->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M->L\\|//\\|//\\|//$%->C~0->L$!->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M->L\\|//\\|//\\|//$%->C~0->L$@->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M->L\\|//\\|//\\|//$%->C~0->L$#->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M^^^^^^666^^^^^^=O

Print the number 1
->L\\|//\\|//\\|//$+->C~0->L$(->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//
->C5~76~8->L$><->C~99~8~7^^^^^^666^^^^^^=M^^^^^^666^^^^^^=O

Operators
The primitive operator  ->  $
    used to access the values that primitives represent
The 666 operator    ->  ^^^^^^666^^^^^^
    used to leach the death expression onto a cell.
    It kills any expression that was there before
The leach operator  ->  ~
    used to leach an arbitrary expression, except the death expression, onto a Cell
The kill operator   ->  =
    used to leach the death expression onto a cell

Expressions
\\|//   ->  drill downwards expression
    -   Open a gate below the gate pointer
//|\\   ->  drill upwards
    -   Open a gate above the gate pointer
jmp l   ->  jump
    -   Jump to the specified label l unconditionally
ijump l -> conditional jump
    -   Jump to the specified label l if the value in Cell 0 in the Cells Region is 0
        or if it contains the Death Expression
Any combination of operators and other expressions
$!  ->  an expression

Region Pointer
In an MB program, there are 2 regions, the Cells and the Layers
A special pointer called the Region Pointer is used to determine
the regions in which an expression will be expressed.
When a program is initialized, the Region Pointer will be pointing to the Cells.
The gates and the primitives are located in the Layers region.
So to access the gates and the primitives, you need to set the Region Pointer to
the Layers Region first, before drilling the gates.

The Cells region is denoted as C
And the Layers region is denoted as L
You use the region pointer operator to change the region
To change to region L, ->L

Chain Leaching
Used with expressions with primitive that have rampage power
For example, A~B~C^^^^^^666^^^^^^=M This is an example of chain leaching with
a primitive represented by $p that goes on a rampage consuming
the cells A, B and C for its own use and killing them.

Expressions
Expressions are everything.
Built in expressions:
The death expression - When this expression occupies a Cell, nothing can be alive in it,
    until an expression is leached onto it, but even then, 5 expressions later, the death
    expression will return

Cells
These are the 15 memory locations in the Cell region that can be used to store expressions.
They are number with hex digits, from 0 to E.
Initially, they are occupied with the death expression, until an expression is leached
onto one. But even then, when an expression is leached onto a Cell, 5 expressions later,
the expression commits suicide and the Death Expression re-occupies the cell

Tokens
The following are all the valid MindBend tokens
-   The Primitives
    !, @, #, +, %, `, &, *, (, ), <>, ><, }, {
-   The operators
    ^^^^^^666^^^^^^=M, ->, $, ~
-   The Special Expressions
    \\|//
-   The Cell Identifiers
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E
-   The Region Identifiers
    L, C


Language Definition

primitive expression = $a
    where a = a primitive identifier

drill expression = \\|//

cell expression = [0-9] | [A-E]

region expression = ->I
    where I = region identifier

leach expression = primitive expression (region expression)* ~ cell expression |
    cell expression ~ cell expression

organism death expression = ^^^^^^666^^^^^^=O

chained leach expression =  cell expression ~ (chained leach expression)+ ^^^^^^666^^^^^^=M

action expression = drill expression | region expression | leach expression | chained leach expression

passive expression = primitive expression | cell expression | label expression

expression = action expression | passive expression


Program Layout
main - driver coordinating the interactions between all the other modules
lexer - Breaking input into the tokens
parser - analysing the tokens, to see if they are arranged correctly according to the Mindbend's
    rules and spitting out an organism expression struct
codegen - Takes an organism expression as input and spits out an LLVM IR

A leach expression from A~B or ~B
    either means copy from location A to location B or from the current primitive to location B

Modus Operandi For the Parser
Accepts a vector of tokens
Spits out a tree like data structure, the Organism Expression, for the codegen
Sample
->L\\|//\\|//\\|//$`->C~0->L$><->C~10~1
will be turned into [Region L, Drill Down, Drill Down, Drill Down , Primitive `, Region C, 
    Tilde, Cell 0, Region L, Primitive ><, Region C, Tilde, Cell 1, Cell 0, Tilde, Cell 1
] by the lexer
The above will then be turned into
Organism Expression - child: (Leach Expression: (left: Primitive `, right: Cell 0))
|
right:
    Organism Expression - child: (Leach Expression: (left: Primitive ><, right Cell 1))
        |
        right:
            Organism Expression - child: (Leach Expression: (left: Cell 0, right: Cell 1))
            |
            right: None
The region and drill expressions didn't get into the tree because they won't be used by
the code generator, but rather, they are just evaluated to check the rules at compile time

Sample
label:hello:->L\\|//\\|//\\|//$+->C~0->L$(->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//
->C5~76~8->L$><->C~99~7~8^^^^^^666^^^^^^=Mjmp:hello:
will be turned into [
    Label hello, Region L, Drill Down, Drill Down, Drill Down, Primitive +, Region C,
    Tilde, Cell 0, Region L, Primitive (, Region C, Tilde, Cell 1, Cell 0, Tilde, Cell 2,
    Region L, Drill Down, Region C, Cell 1, Tilde, Cell 3, Cell 2, Tilde, Cell 4, Region L,
    Drill Down, Region C, Cell 3, Tilde, Cell 5, Cell 4, Tilde, Cell 6, Region L, Drill Down,
    Region C, Cell 5, Tilde, Cell 7, Cell 6, Tilde, Cell 8, Region L, Primitive ><, Region C,
    Tilde, Cell 9, Cell 9, Tilde, Cell 7, Tilde, Cell 8, TripleSixEqM, Jump hello
]
The above will then be turned into
Organism Expression - child: (Label Expression: (name: hello))
|
right:
    Organism Expression - child: (Leach Expression: (left: Primitive +, right: Cell 0))
    |
    right:
        Organism Expression - child: (Leach Expression: (left: Primitive (, right: Cell 1))
        |
        right:
            Organism Expression - child: (Leach Expression: (left: Cell 0, right: Cell 2))
            |
            right:
                Organism Expression - child: (Leach Expression: (left: Cell 1, right: Cell 3))
                |
                right:
                    Organism Expression - child: (Leach Expression: (left: Cell 2, right: Cell 4))
                    |
                    right:
                        Organism Expression - child: (Leach Expression: (left: Cell 3, right: Cell 5))
                        |
                        right:
                            Organism Expression - child: (Leach Expression: (left: Cell 4, right: Cell 6))
                            |
                            right:
                                Organism Expression - child: (Leach Expression: (left: Cell 5, right: Cell 7))
                                |
                                right:
                                    Organism Expression - child: (Leach Expression: (left: Cell 6, right: Cell 8))
                                    |
                                    right:
                                        Organism Expression - child: (Leach Expression: (left: Primitive ><, right: Cell 9))
                                        |
                                        right:
                                            Organism Expression - child: (Chained Leach Expression: left: (Leach Expression: (left: Cell 9, right: Cell 7)), right: Cell 8)
                                            |
                                            right:
                                                Organism Expression - child: (Jump: (name: hello))
                                                |
                                                right: None


# Implementing expression and gate lifetimes
-   Start counting the lifetimes of expressions as they are created
-   An expression whose lifetime is over should not be able to be accessed
-   Jumping should not overthrow the expression's lifetimes
## Considering compile time
Every expression has a time to live.
After the creation of every expression, every already living expression's time to live reduces
For instance,
->L\\|//\\|//\\|//label:i:"pos 1" $+->C~0 "pos 2" ->L$`->C~1 "pos 3" ->L "pos 4" $><->C~3jmp:i: "pos 5"

### At pos 1,
-   The 3 gates have been open and their time to stay open begins from 5
### At pos 2
-   2 active expressions are created
-   The gates TTSO reduces to 3
-   An expression is leached onto Cell 0 and its TTL starts from 5
### At pos 3
-   3 more active expressions have been created
-   The gates TTSO reduces to 0
-   The gates are now closed
-   The expression in Cell 0's TTl reduces to 2
-   An expression is created and leached onto Cell 1 and its TTL starts from 5
### At pos 4
-   1 active expression is created
-   Expression in Cell 0s TTL reduces to 1
-   Expression in Cell 1s TTL reduces to 4 
### At pos 5
-   A primitive access is attempted which should fail because the gates are closed
But consider the scenario where the gates aren't closed.
In such a scenario after obtaining the primitive, switching to the Cell region and leaching the
resulting expression onto Cell 3, let's assume that the gates now just closed. After the jmp expression
execution continues from label i. But how can you determine that that primitive access is valid
after the 2nd jump or the 3rd or the nth. You can't. It can only be evaluated/determined at runtime.

## Considering run time
The same instance from the consideration above:
->L\\|//\\|//\\|//label:i:"pos 1" $+->C~0 "pos 2" ->L$`->C~1 "pos 3" ->L "pos 4" $><->C~3jmp:i: "pos 5" 
### At runtime, 
*   A number representing the gates state, CGS, is initialized to 0, meaning no gates are open
    CGS - 0: no gates are open, 1: 1 open, 2: 2 open, 3: 3 open
*   A number representing the time to stay open of the gates, TTSO, is initialized to 5
*   A number representing the current region, CR, is initialized to 0, representing the Cells region
    CR - 0: Cells, 1: Layers
*   15 memory locations representing the 15 Cells are initialized
*   A table mapping cell locations to the time to live of their host expressions, TTL table, is initialized
*   A table mapping cell locations to primitives numbers representing a function, the Leached Primitive Function
    table, LPF table, is initialized
### At pos 1
-   ->L changes CR from 0 to 1
    The TTSO of the gates is compared to 0 and found to be equal to 0
    Since the TTSO is already equal to 0, it isn't reduced
    The CGS is set to 0, since the TTSO is 0
    The TTL table is checked to see if there are any expression stored in Cells
    Since the TTL table is empty, nothing is altered
-   1 \\|// is encountered
    The CR is compared with 1 (Layers)
    If it isn't equal to 1, program should crash with an error
    But it is equal to 1, so continue
    The CGS is compared to the number 3, the highest number it can be
    If it's equal to 3, then don't alter it
    Since it's not equal to 3, increase it by 1, then it becomes 1
    The expression type is checked and found to be a drill expression
    If it wasn't a drill expression, the TTSO would have been compared to 0 and reduced by 1 if it wasn't 0
    Since it's a drill expression, the new CGS is compared to 3
    If it's equal to 3, set TTSO to 5
    The TTL table is checked and found to be empty, so nothing is altered in it
    If the TTL had values, they would have been reduced, since drilling is an active expression
-   2 more \\|// changes the number representing the gates state from 1 to 2 to 3, meaning
    All the checks that took place after the 1st one take place after the other 2
-   The next block is put in under the label i
### At pos 2
-   A primitive access is attempted
    The CR is compared to 1 (Layers)
    If it's not equal to 1, the program crashes with an error
    But since it's equal to 1, continue
    The CGS is compared to 3 (all open)
    If it's not equal to 3, the program crashes with an error
    But since it's equal to 3, continue
-   At -> C, CR 0, meaning the Cells region is now the current region
    The TTSO is compared to 0, the lowest it can be
    Since it's not 0 reduce it by 1, from 5 to 4
-   At ~0, the target cell is found to be Cell 0
    The TTL table is checked to see if there are any values in it
    If there were values in it, they would have been reduced
    Since there are no values in the TTL, nothing is altered
    An entry is created for Cell 0 in the TTL table, mapping 0 to 5, meaning the expression in Cell 0
    commits suicide after 5 active expressions have been created
    The gates TTSO is compared with 0
    Since it's not 0, it reduces by 1, from 4 to 3
    The gates TTSO is compared with 0 again
    If it is 0, the CGS is set to 0, all closed
    The left side of the leach expression is checked and found to be a primitive
    The primitive is checked and found to represent an integer value, not a function
    If it represented a function, then the cell number will be associated with that function
    Since it represents an integer, the integer is simply stored in the memory location that
    Cell 0 represents
### At pos 3
-   The CR is set to 1, meaning now in the Layers region
    The TTSO is compared with 0
    If it's 0, don't do anything
    Since it's not 0, the value is reduced by 1, from 3 to 2
    The new TTSO is checked and found to be 2
    If it was 0, the CGS would have been set to 0, all closed
    Since it's not 0, do nothing
    The TTL table is checked and found to have entries
    Every value in the TTL table, which at this point is only the Cell 0 mapping, is reduced by 1
    So, the value mapped to from 0, representing expression in Cell 0's lifetime is reduced from 5 to 4
-   The primitive with symbol ` is attempted accessed
    The CR is checked and found to be 1
    The CGS is checked and found to be 3
-   The number representing current region changes to 0, meaning current region is now Cells region
-   Region changing is active
    All entries in the TTL table are reduced by 1.
    The entry with the key of 0's value changes from 4 to 3
    TTSO of gates reduces from 2 to 1
-   The currently accessed primitive is leached onto the Cell 1.
    The memory location representing Cell 1 is set to the primitive value
    Since leaching is an active expression, all entries in the TTL table are reduced by 1
    The entry with the key 0 reduces from 3 to 2
    An entry is created in the TTL table with a key of 1 and a value of 5
    TTSO of gates reduces from 1 to 0
    The new TTSO is checked and found to be 0. The gates are then closed, that is,
    the gates state number is set to 0, meaning 0 gates are open
### At pos 4
-   The number representing the current region is set to 1
-   The TTL entries are reduced by 1
    The entry with the key of 0's value changes from 2 to 1
    The entry with the key of 1's value is is reduced from 5 to 4
    The TTSO is checked and found to be 0, so it isn't altered
### At pos 5
-   Primitive access is attempted
    The current layers number is checked and found to be 1
    The gates state number is checked and found to be 0, all gates are closed
    The program exits with an error
Consider the scenario where the gates aren't closed.
In such a scenario after obtaining the primitive, switching to the Cell region and leaching the
resulting expression onto Cell 3, let's assume that the gates now just closed. After the jmp expression
execution continues from label i. The attempt to access primitive + in the beginning of pos 2 will still
fail because the number representing the current gates state will be 0.
So checking at runtime is immune to jmp expressions


## Flows
-   access Validation
-   State updates
Note: 
-   In all validation routines, a return value of 0 is a success and a return value of 1
is an error
-   All functions that return values return 32 bit values

### After every \\|//
Carry out the drill gate routine
Compare the result of the DG routine with 0
If it is 0
    Continue
If it isn't 0
    End in fail
Carry out the state update routine with update TTSO set to 0

### After every ~
There are 3 types of leach expressions (From the perspective of the codegen)
Store, copy and massacre consumption (function call)
When a primitive is leached onto a cell, it's stored there
When a cell is leached onto another cell, it's copied there
When a cell is chain leached onto multiple cells and ends with a 666 Massacre, it's a consumption
Only function primitives can go on massacres

The arg on the left should be checked to see whether it is a primitive or cell
If it is a primitive,
    Carry out the PA Routine
    If the result of the routine is 0
        Continue
    If the result of the routine is non 0
        End in fail
    Check what type of primitive it is
    If the primitive represents a value like an integer,
        Store it in the memory location representing the target Cell
    If the primitive represents a function
        Store the primitive index number in the memory location representing the target Cell
    Carry out the State Update Routine
    Store 5 in the TTL cell corresponding to the target Cell
If it is not a primitive, then it's a cell
    Check if the leach expression is a chained one
        If it is, then it's the calling of a function
            Carry out the FV Routine
            Compare the result of the routine with 0
            If it is 0
                Continue
            If it is not 0
                End in fail
            Depending on the function, iterpret the args (cell numbers down the chain) as the args
            Call the function
            Set all entries in the TTL table associated with the Cell numbers to 0
            Carry out the State Update Routine
            Store the result in the memory location corresponding to the last arg's Cell
            Store 5 in the TTL table cell which corresponds to the last arg's Cell
        If it isn't, then it's a copy
            Carry out the ELV routine for the src Cell
            If the result of the routine is 0
                Continue
            If the result of the routine is 1
                End with code 1
            Get the memory address associated with the src Cell
            Retrieve the value from there and copy it into the the memory location of the target Cell
            Carry out the State Update Routine
            Create a new entry in the TTL table mapping the target Cell's number to 5

#### Function args interpretation
Addition: A~B~C
Add A to B to C and store the resulting expression in C
Subtraction: A~B~C
Perform A - B - C and store the result in C
Output: A~B~C~D~E
Take A * 10 + B, C*10 + D and E as ascii and print them 1 character at a time

### After every ->
Check the target region
If it is Cells
    Set the CR to 0
If it is Layers
    Set the CR to 1
Carry out the SU Routine with the update TTSO set to 1 (or any non-zero value)

### Primitive Access Routine
#### Purpose
-   Validate primitive access
#### Args
CR, CGS
#### Flow
The CR is compared to 1
If it is not equal to 1
    Print an error
    Return 1
If it is equal to 1
    The CGS is compared to 3
    If it is not equal to 3
        Print an error
        return 1
    If it is equal to 3
        Return 0

### Cell Access Routine
#### Purpose
-   Validate primitive access
#### Args
CR
#### Flow
The CR is compared to 0
If it is not equal to 0
    Print an error
    Return 1
If it is equal to 0
    Return 0

### State Update Routine
#### Purpose
-   Reduces the gates' TTSO after active expressions, except the drill expression
    The update TTSO arg is used to stop the routine from altering the TTSO when a drill expression's
    code is being generated
-   Reduce leached cells TTL by altering the TTL table
#### Args
TTSO, TTL table, CGS, update TTSO, Cells Region
#### Flow
Check if the update TTSO argument is 0
If it is not 0
    Check if the TTSO is 0
    If it is 0
        Don't alter it
        Continue
    If it is not 0
        Decrement 1 from the TTSO
        Compare the new value with 0
        If the new value is 0
            The CGS should be set to 0
        If the new value isn't 0
            Continue
If it is 0
    Continue
Check the TTL table to see if there are any non 0 values in it (0 signifies dead expressions)
If there aren't non 0 values in it
    Don't alter it
    Continue
If there are non 0 values in it
    The ones that aren't equal to 0 should be reduced by 1
    Check the new value after reducing to see if it is equal to 0
    If it is
        Set the corresponding Cell in the Cells Region to -1
    If it isn't
        Continue
Return nothing

### Drill Gate Routine
#### Purpose
-   Validate drill access
-   Update the CGS after drills
#### Args
CR, CGS, TTSO
#### Flow
Check if the CR is 1
    If it isn't 1
        Print an error
        return 1
    If it is 1, the program should continue
    Check if the CGS is 3
        If it isn't 3,
            Increase by 1
            Check if it is now 3
            If it is 3
                Set the TTSO to 5
            If it is not 3
                Continue
            Return 0
        If it is 3
            Don't alter CGS
            Return 0

### Function Validation Routine
#### Purpose
-   To check if the number in a memory cell corresponds to a valid function primitive index
#### Args
Cell number, Cells Region, TTL table
#### Flow
Get the memory address representing the Cell's TTL entry in the TTL table
Load the number in the TTL table
Compare the number with 0
If it is 0
    Print an error (Probably not)
    Return 1
If it is not 0
    Continue
Get the memory address representing the Cell in the Cell Region
Load the number in the cell
Compare the number with each of the primitive indexes that correspond to primitive functions
    (input, output, addition, subtraction)
If there is no match
    Print error (Probably not)
    Return 1
If there is a match
    Return 0

### Expression Life Validation Routine
#### Purpose
-   To check if the expression associated with a Cell is still alive
#### Args
TTL table, Cell number
#### Flow
Check the cell in the TTL table associated with the Cell
If the value there is 0
    Print error (Probably not)
    Return 1
If the value there is 1
    Return 0


### Data Landscape Initialization Routine
Allocate space for a number and store 0 in it, representing the CGS
Allocate space for a number and initialize it to 5, representing the TTSO
Allocate space for a number and initialize it to 0, representing the CR
Allocate a contiguous region of space to hold 15 numbers, all initialized to -1, representing the Cells Region
Allocate a table mapping, all initialized to 0, representing the TTL table

### Table Mappings
A table will be a contiguous memory region that can hold 15 numbers
Each location in the memory region will hold a mapping for a cell
The memory location with the index A will hold the value for a cell A

# CodeGen Layout
Need seperate LLVM Functions for the routines
