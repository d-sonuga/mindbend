# Overview
Mindbend is an esoteric language for real programmers only.
For a description of the language, see mindbend.md
MindBend.md, on the other hand, is just a bunch of raw thoughts that came and went while
developing the language and the compiler. Some of the stuff there don't correspond with what
is actually in the code.

## The Mindbend Philosophy
*   Only the strong survive
*   Optimize as you write
*   Comments are for whimps

## The Code
If you want to take a look at the code, I must warn you that I somewhat fell in love
with trees and boxes before writing it, so some parts which are meant to be straightforward
are unecessarily complicated.
Some of the language was still ambiguous when I started with the compiler, so some things are
way out of place.

## Tests
The tests aren't extensive at all. They're barely there.
To run them, you need to set --test-threads=1 so they won't interfere with each other.