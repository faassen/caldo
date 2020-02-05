===============
Instruction Set
===============


------------------------
Implemented instructions
------------------------

General behavior
================

If the stack is empty when an instruction pops the top, increase failure count
and pass to next instruction.

If the stack grows beyond a certain size, it's compacted: the bottom half
is thrown away.

Failures increase the failure count of the processor. The failing
instruction is skipped. If the failing instruction removes items
from the top of the stack, the stack is still affected. No new values
will be placed on the stack.

TRUE is the number 0xFFFFFFFF.

FALSE is the number 0.

Control Flow
============

Introduction
------------

If the end of a gene is reached, return to the calling gene on the callstack.
If the call stack is empty, go to the beginning of this gene.

JF (a b --) Jump Forward
------------------------

If `a` is non-FALSE, jump to `pc + b` on gene. If `a` is FALSE, don't jump. If
`b` is 0 don't jump either.

If jump would go beyond end of gene this is a failure.

JB (a b --) Jump Backward
-------------------------

If `a` is non-FALSE, jump to `pc - b` on gene. If `a` is FALSE, don't jump. If
`b` is 0 don't jump either.

If jump would go beyond start of gene this is a failure.

Lookup (a -- b) Look Up Gene
----------------------------

Look up gene id for `a`. Place the gene id on the top of the stack.

Call (a --) Call Gene
----------------------

Take `a` as gene id. Find matching gene for it. If gene cannot be found this is
a failure. If gene can be found, transfer to this gene, set `pc` to 0.

One the end of that gene is reached, transfer is moved back to the current
gene, if it still exists.

Arithmetic
==========

Introduction
------------

Arithmetic overflow or underflow results in a failure.

Add (a b -- c) Add
------------------

Add `a` and `b` and place result on top of stack.

Sub (a b -- c) Substract
------------------------

Substract `b` from `a` and place result on top of stack.

Mul (a b -- c) Multiply
-----------------------

Multiply `a` with `b` and place result on top of stack.

Div (a b -- c) Divide
---------------------

Divide `a` with `b` and place result on top of stack. Division by zero results
in a failure.

Comparison operators
====================

Introduction
------------

Eq (a b -- c) Equal
-------------------

If a is equal to b, place TRUE on top of the stack, otherwise FALSE.

Ne (a b -- c) Not Equal
-----------------------

If a is not equal to b, place TRUE on top of the stack, otherwise FALSE.

Gt (a b -- c) Greater Than
--------------------------

If a is greater than b, place TRUE on top of the stack, otherwise FALSE.

Lt (a b -- c) Lesser Than
-------------------------

If a is lesser than b, place TRUE on top of the stack, otherwise FALSE.

Logic operators
===============

And (a b -- c) And
------------------

If `a` and `b` are both non-zero, place TRUE on top of the stack, otherwise
FALSE.

Or (a b -- c) And
------------------

If `a` or `b` or both are non-zero, place TRUE on top of the stack,
otherwise FALSE.

Not (a -- b) Not
----------------

If `a` is non-zero, place TRUE on top of the stack, otherwise FALSE.

Stack manipulation
==================

Dup (a -- a a)
--------------

Top of stack.

Drop (a -- )
------------

Drop top of stack.

Swap (a b -- b a)
-----------------

Swap top of stack.

Over (a b -- a b a)
-------------------

Place copy of one below top of stack on the top of the stack.

Rot (a b c -- b c a)
--------------------

Rotate the top of the stack.

-----------------
To be implemented
-----------------

Call stack overflow compaction.

Execution costs ATP.

Gene construction
=================

Read (a b -- c)
---------------

Read index `a` of gene id `b`. Place value there on stack.

If gene id `b` does not refer to a gene, failure.

If index `a` does not exist on gene, failure.

Create ( -- a )
---------------

Create a new gene. `a` is the gene id of the newly created gene.

Write (a b -- )
---------------

Write value `a` to the end of the gene with gene id `b`.

If gene id `b` does not exist, failure.

Ideas
-----

Can these also be used to read from input queues? Write is like a queue, but
read isn't. Unless we introduce read heads we can't really track where we read.
Or do we want to arbitrarily read from a "sensor strip" too?

