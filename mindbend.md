# Overview
In mindbend, everything is an expression.
And by expression, I don't mean the 1 + 1 you hear people calling expressions in other languages.
In mindbend, that's not an expression. That's not even possible.
Expressions are living things.
And they dwell in an environment which, in mindbend, is called the Data Landscape.
And they die, mostly by committing suicide.
Every mindbend program is simply the creation of expressions and their interation with their
environment.
The purpose of this is to introduce mindbend and to explain its concepts

# The Concepts
*   Data Landscape
*   Expressions

## Data Landscape
This is where everything in your program takes place. You can visualize the Data Landscape as
a weird sort of physical environment with only 2 rooms. In 1 room, you can store things in a limited
amount of space. But by default, there is nothing in the room. By default you have nothing. And that's
where the other room steps in. Everything you want can only be gotten from the other room.
            Empty Room          Room Full of Stuff
            ------                  -----
            |    |                  |||||
            |-----                  |----
To do anything useful, or anything at all, everything you need must be gotten from the room Full of Stuff.
You may be thinking that why not just move to the Room Full of Stuff and forget the empty one.
Well, you can't do that because it's not allowed (don't ask me why; it's life).
Any action you want to take can only be taken in the Empty Room. But everything you need to take any action
at all is in the Room Full of Stuff. Take a moment to languish the predicament.
Aha! Now, you have a light bulb moment. You realize that you can simply do the following:
-   Go to the Room Full of Stuff
-   Get stuff you need
-   Go back to the Empty Room
-   Use the stuff
And you're happy. Problem solved.
No.
Sorry bro, but life strikes again.
After a few trials, you realize the following rules govern the inter-room movement
-   You can only move 1 thing at a time
-   Everything that you move magically has a ridiculously close expiry date 
-   When something you move has expired, it self destructs
-   The more things you move, the faster the things you have moved expires
Wonderful situation, don't you think so?
Well, this is the Data Landscape.
Only, the rooms are called Regions.
The "Empty Room" is the Cells Region and the "Room Full of Stuff" is the Layers Region.
Everything in a mindbend program takes place within these 2 regions.
A mindbend program is the interaction of expressions and The Regions.

### Cells Region
In here, there is something called a Cell. It's the stuff expressions need to survive (remember
expressions are living things). There are 15 of them in total, numbered in hex digits from 0 to E.

### Layers Region
In here, there are 2 things you need to be aware of:
-   The primitives
-   The gates

#### The Primitives
The primitives are the stuff you need to do anything useful. Anything at all.
You may be familiar with statements like "a = 1" from other languages, but no such thing exists
in mindbend. If you need to use the number 1 in a computation of any sort, you need a primitive.
The primitives are just like the useful things in the Room Full of Stuff. You need to get them to the
Cells Region to do anything with them.
All primitives are identified by symbols, or a number called the primitive index.
The Primitive Table
Symbol  ->  Index   ->  Primitive
"!"            D             1
"@"            C             2
"#"            B             3
"+"            A             4
"%"            9             5
"`"            8             6
"&"            7             7
"*"            6             8
"("            5             9
")"            4             0
"<>"           3           input
"><"           2           output
"}"            1           addition
"{"            0           subtraction

#### The Gates
The gates are bane of your existence. They must be opened, or, more correctly, `drilled` through.
There are 3 of them. You must drill through them to access the primitives. They automatically close at certain
intervals. Then you must drill through them again to access the primitives.
    Layers (Now you know why it's called `Layers` Region):    
        === - gate 1
        === - gate 2
        === - gate 3
     !@#$%&*()+{} - The Primitives, at the bottom of the layers
    
## Expressions
The main characters.
Living things.
Interact with their environment to make things happen.
They must fend for themselves, or they get killed, massacred.
They die, commit suicide.
They reproduce, and die in the process.
At the beginning of every mindbend program, all the cells in the Cells Regions are occupied
by a special expression, the Death Expression, the only expression that doesn't die.
All expressions are equal, but some are more equal than others.
A complete mindbend program, being nothing but a cluster of expressions is itself an expression: the organism expression.
And every program must end with the death of the organism expression.

# Code
At the start of every mindbend program:
-   You're in the Empty Room, the Cells Region
-   All the gates in the Layers Region are closed
Also note:
-   Whitespace is not allowed (don't ask why).

Consider the following:

->L\\|//\\|//\\|//$`->C~0->L$%->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~97~A8~B9~B~A^^^^^^666^^^^^^=M^^^^^^666^^^^^^=O

The above is a program that prints the letter 'A'.
It gets the ascii number for 'A', 65, and prints it with the output primitive.
Now, let's take it step by step

->L - This is a region expression. Remember that at the start of the program, you're in the Cells Region.
    Well, what this simply means "move to the Layers Region". After this expression, you're now in the Layers
    Region.

\\|// - This is a drill expression. You use it to drill through gates. You can see that there are 3 of them.
    All 3 are closed initially. Each '\\|//' has the effect of opening a single gate.

$`->C~0 - This is a leach expression. This single expression has the effect of creating a new expression
    that has a primitive identified by the symbol '`', the primitive 6, and *leaching* it onto Cell 0.
    The newly leached expression becomes the new host of the cell. If it isn't leached, it doesn't survive.
    It dies without ever being useful.
    The $ is the primitive operator, used to obtain primitives by either their symbol or primitive index.
    The region expression, ->C, means switch to Cells Region. It is necessary because the cells exist only
    in the Cells Region and you have to be in that region to access them. As a matter of fact, you can
    have an arbitrary number of region expressions between $` and ~0.

->L - Another region expression. It's necessary because you need to be in the Layers Region to access any
    primitive and you need more primitives to print the letter 'A'

$%->C~1 - Another leach expression. The primitive 5 is obtained, the region is changed to the Cells Region
    and a newly created expression holding the primitive 5 is leached onto Cell 1

0~2 - This is a leach expression. But instead of leaching primitives, it's a cell. This has the effect of
    reproducing the expression in Cell 0 and leaching the result of that onto Cell 2, killing the expression
    in Cell 0 in the process.
    Region expressions aren't allowed in this kind of leach expression, so 0->C~2 is invalid.

->L - Another region expression

\\|// - A drill expression.
    You may be wondering why a drill expression is here again. Haven't the gates already been drilled
    through earlier. Yes, but now their closed. All of them. And you have to drill through all 3 to
    regain access to the primitives.
    Now, the question is why did they close?
    You see, expressions can be grouped in 2 groups: active and passive.
    After the gates have been opened, once 5 active expressions have been created, they close.
    All 3 of them.
    And you need them opened to access the primitives.
    The active expressions encountered so far are the region expressions: ->L, ->C, the leach expressions:
    $%->~1, 0~2, and the drill expression. The drill expression is the only active expression that
    doesn't count for the gates open lifetime.
    $`->C~0->L$%->C~10~2->L
    ->C 1 active expression created
    ~0  1 active expression created
    ->L 1 active expression created
    ->C 1 active expression created
    ~1  1 active expression created
    0~2 1 active expression created
    ->L 1 active expression created
    7 in total.
    So the gates are closed.

->C1~32~4 - Switch to cell, reproduce expression in Cell 1 and leach it onto Cell 3, reproduce
    expression in Cell 2 and leach it onto Cell 4 (killing the expressions in both Cell 1 and 2).
    But why are we doing this?
    Why don't we just finish drilling through the gates, get the output primitive and print the letter.
    Well, you can't.
    You remember the auto-closing of the gates.
    Well, something similar applies to expressions.
    Once an expression is leached onto a cell, 5 active expressions later, it commits suicide.
    It and all it's usefulness is no longer accessible to you.
    You can't save the expression. It's doomed.
    But you can save it's usefulness, and you do that by reproduction, over and over, until you
    have everything you need to finally do what you want to do.
    All active expressions count, even the drill expressions.
    
->L\\|//->C3~54~6->L\\|// - Switch to Layers Region, drill. At this point, 3 expression have been created
    since the expression in Cell 3 was leached onto Cell 5. It's usefulness must be reproduced and saved
    so it won't die with it. So, switch back to Cells Region, reproduce and leach, saving our precious
    6 and 5, which we need to print the letter 'A', then switch again to the Layers Region to drill

->C5~76~8->L$><->C~97~A8~B - After this delicate dance of reproducing, leaching, drilling, and switching
    regions, the output primitive, with symbol '><' is finally obtained and leached onto Cell 9, then
    out precious 6 and 5 are saved again in the same dance, 7~A8~B

9~B~A^^^^^^666^^^^^^=M - This is a chain leach expression. The expression in Cell 9, which holds an
    output primitive goes on a massacre consuming the expressions in Cell B, Cell A in that order.
    Currently, the expression in Cell B holds primitive 6 and the expression in Cell A holds primitive 5
    The output primitive takes it as 65 and prints it.
    At the end of the massacre, the expression that went on killing kills itself.
    The resulting expression from any massacre is leached onto the final cell in the chain, which in this
    case is supposed to be A. But since the output primitive doesn't produce any expressions, the new host
    in Cell A is undefined.
    The ^^^^^^666^^^^^^=M signifies the end of the massacre.
    Please note that only primitives with rampage power can go on massacres. A list of them are below.
    Region expressions are not allowed between them. 9->C~B~A^^^^^^666^^^^^^=M is invalid.

^^^^^^666^^^^^^=O - This signifies the death of the organism expression. It must be the last token in
    every mindbend program

## Other Expression Types
label:x: - The label expression
    A passive expression which is used to label a segment of expressions
jmp:x: - The jump expression
    An active expression (its use results in the reduction of gates and leached expressions' lifetimes)
    used to jump to the segment labelled x
ijmp:x: - The conditional jump expression
    Just like the jump expression, but only jumps when the expression in Cell 0 has a primitive 0

# Primitives With Rampage Power
*   input - Used like so
        2~3^^^^^^666^^^^^=M
        where Cell 2 holds an expression with primitive input,
            Cell 3 ends up holding the ascii value of the character the user inputted

*   output - Used like so
        2~3~4~6^^^^^^666^^^^^^=M
        where Cell 2 holds an expression with primitive output
            the ascii value of [3]*10+[4] and [6], is what gets printed
                where [I] means the primitive of the expression in I
            The cells down the chain are taken in 2s until they can't
            Cell 6's content at this point is unknown. It can be anything,
            since the output primitive doesn't produce any expressions

*   addition - Used like so
        2~3~4^^^^^^666^^^^^^=M
        where Cell 2 holds an expression with primitive addition
        primitives in Cell 3 and 4 are added and the resulting expression is leached onto Cell 4
    
*   subtraction - Used like so
        2~3~4~5~6~7^^^^^^666^^^^^^=M
        Where Cell 2 holds an expression with primitive subtraction
        evaluated as [3] - [4] - [5] - [6] - [7] and the resulting expression is leached onto Cell 7
            where [I] means the primitive of the expression in I

An expression holding a primitive apart from the above mentioned can't go on a massacre

# Language Definition
primitive expression = $a
    where a = a primitive identifier

cell expression = [0-9] | [A-E]

label expression = label:y:

drill expression = \\|//

region expression = ->I
    where I = region identifier

leach expression = primitive expression (region expression)* ~ cell expression |
    cell expression ~ cell expression

chained leach expression =  cell expression ~ ((chained leach expression)+ | cell expression) ^^^^^^666^^^^^^=M

jump expression = jmp:y:

consditional jump expression = ijmp:y:

organism death expression = ^^^^^^666^^^^^^=O

active expression = drill expression | region expression | leach expression | chained leach expression

alpha passive expression = primitive expression | cell expression

beta passive expression = label expression

passive expression = alpha passive expression | beta passive expression

expression = active expression | passive expression | organism death expression