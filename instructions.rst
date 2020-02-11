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

Gene ( -- a )
-------------

Create a new gene. `a` is the gene id of the newly created gene.

Write (a b -- )
---------------

Write value `a` to the end of the gene with gene id `b`.

If gene id `b` does not exist, failure.

Ideas
-----

Can these also be used to read from input queues? Write is like a queue, but
read isn't. Unless we introduce read heads we can't really track where we read.
Or do we want to arbitrarily read from a "sensor strip" too, i.e. an
input array.

The wall strength could be in an input array.

A Port System
==============

A cell can maintain ports. These ports are the way it interacts with the
outside world. A port can be used to ingest molecules and expell them to the
environment. A port may also be connected to another port of a neighboring
cell. This can allow a cell to ingest and emit materials.

A port can also be used for communication: ports have associated queues,
one in each direction. Values can be placed on the queue and read from
the other end.

Cell
====

Cell ( -- a)
------------

Create a new cell. `a` is the cell id of the newly created cell.

Idea: create new cell at ort?
Wall ( -- )
-----------

Strengthen the cell wall.

OpenPort ( a -- b )
-------------------

Make a new port with a as port lookup. Return port id.

ClosePort ( b -- )
------------------

Close a port with port id.

LookupPort ( a -- b)
--------------------

Lookup port with port lookup. Return port id.

MoveGene (a b -- )
------------------

Move gene with gene id a into cell with cell b.

Fails if gene id or cell id does not exist.

Idea: move into port?

Metabolism
==========

Ingest ( a -- )
---------------

`a` is the element id. Element is looked up and ingested from the world
immediately around the cell.

Expell ( a -- )
---------------

`a` is the element id. Element is looked up and ejected into the world around
the cell.

Connect (a b -- c)
------------------

Given cell id and port lookup, return port id of neighboring cell. This
connection can be broken if neighboring cell is further distant.

ExpellPort (

