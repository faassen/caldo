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

Lookup (a -- gene_id) Look Up Gene
----------------------------------

Look up `gene_id` for `a`. Place the gene id on the top of the stack.

Call (gene_id --) Call Gene
---------------------------

Find matching gene for `gene_id`. If gene cannot be found this is a failure. If
gene can be found, transfer to this gene, set `pc` to 0.

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

Gene construction
=================

GeneCreate ( a -- gene_id)
--------------------------

Create a new gene with value a as the first value.

GeneRead (gene_id a -- b)
-------------------------

Read index `a` of `gene_id`. Place value there on stack.

If `gene_id` does not refer to a gene, failure.

If index `a` does not exist on gene, failure.

GeneWrite (gene_id a -- )
-------------------------

Write value `a` to the end of gene.

If gene id does not exist, failure.

GeneComplete (gene_id --)
-------------------------

XXX do we want this? It makes various systems harder
to write. On the other hand is supports compilation.

Finish writing the gene. After this is it is ready to
receive processors. It cannot be modified anymore.

GeneDestroy (gene_id --)
------------------------

Do we want this? This would break the gene into its component
materials.

Processor
---------

ProcCreate (gene_id a -- )
--------------------------

Start a processor on gene with index a.

Pun not intentional.

ProcDestroy
-----------

This processor is removed.

Homeostasis
============

ChemAmount (chem -- a)
----------------------

Find chemical. Return the amount of this chemical in this cell.

WallAmount (-- a)
-----------------

Find the amount invested in the integrity of the cell, i.e. the wall.

Organelle System
================

There are a number of organelles in the system. These can be used
to:

* induce chemical reactions.

* communicate with neighboring cells.

* transfer genes and chemicals to neighboring cells.

* ingest or expel chemicals with the environment.

* maintain the cellular integrity, the "wall".

The various organelles just "exist", so that the cell has access
to this equipment. It may however not be able to actually use them,
as this requires sending the right outputs for given inputs.

Chemical reactions may be otherwise bounded by external circumstances, such as
light, temperature, etc.

An organelle can also be used to connect to neighboring cells. How
many of such connection organelles exit depends on the system; one
for each direction, for instance, or only a single one per cell.

A connection can be attempted with an instruction. A connection organelle can
also be used to create a child cell.

Once a connection is established, the input port is connected the other cell's
output port and vice versa. Communication can flow through this channel.

A connection can be opened or closed by the cell for chemicals and genes. An
open connection can be used to transfer these to another cell.

LookupOrg (a -- org_id)
-----------------------

Lookup near organelle by org identifier.

Input (org_id -- a)
-------------------

Input one value from org_id.

HasInput (org_id -- a)
----------------------

Return TRUE if there is input on port org_id.

Output (org_id a --)
--------------------

Output value to org_id.

OpenGene (org_id --)
--------------------

Open org for gene input. Only makes sense for communication ports.

OpenChem (org_id --)
--------------------

Open org for chemical input/reaction. If a chemical reaction organelle is
closed, it won't actually perform the reaction even though input/output might
cause it to. If an external organelle is closed, expel and ingest won't
work.

CloseGene (org_id --)
---------------------

CloseChem (org_id --)
---------------------

IsOpenChem (org_id -- a)
------------------------

IsOpenGene (org_id -- a)
------------------------

Expel (org_id chem -- )
-----------------------

Transfer chemical (identified in chem space) through communication port or
to environment.

Ingest (org_id chem -- )
------------------------

Ingest chemical (identified in chem space) through organelle. This only
works for environment organelles.

Cell (org_id -- )
-----------------

Create a new cell at a communication port. It does nothing for other
ports. The new cell starts connected through this communication port,
and everything is open.

MoveGene (gene_id org_id -- )
-----------------------------

Move gene with gene_id through port identified by org_id. If it's not a
communication port, nothing happens, and also not if it's not connected to
other cells.
