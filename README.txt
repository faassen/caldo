Caldo primigenio
================

Primordial Soup on the web.

Technology choices:

* We build it in Rust entirely.

* We build an interpreter.

* We don't try to directly generate web assembly.

* We use a stack machine architecture.

* We use a web browser with web assembly where needed to drive the UI and
process.

A cell is composed of genes. Genes are sequences of instructions. These
instructions consist of 3 byte triplets. When a gene is executed for the first
time, a genotype to phenotype mapping takes place. Instructions associated with
triplets are looked up in instruction space. Only those triplets that follow
the special QUOTE instruction are left as a special number instruction.

Genes are identified by a 3 byte triplet as well. Lookups of genes is by
triplet. This lookup is unreliable - different genes at approximately the same
distance may match.

A gene's activation can be increased and decreased by itself, as well as other
genes. This is done by ++ to increase and -- to decrease. If a gene is more
activated, it executes more quickly than a gene that is less activated. A gene
can measure its own activation and the activation of other genes as well.

Cells live on a simple two dimensional grid. Each cell is connected to 4 of its
neighbors, N, W, S, E. A grid location does not need to contain a cell.

A gene may cause a new connected empty cell to be created (CELL). The top of
the stack indicates the cardinal direction. This takes energy and materials. A
success (FFFFFF) or failure message (000000) is placed on the top of the stack.

A gene may cause a gene to be copied into another connected cell (COPY). This
takes energy and materials. This can fail if the other cell has a barrier
and has not opened it. Success/failure is placed on the top of the stack.

A gene may cause energy and materials to be transferred into another cell. The
cell has a number of reserve pools. The pool and amount are taken from the top
of the stack. This can fail if the other cell has a barrier and has not opened
it. Success/failure is placed on the top of the stack.

A gene may send messages to another cell (SEND from top of stack), to a special
"messages queue". A gene can also read a message onto the top of its stack
(RECEIVE). The messages pool is shared by all genes. Messages are not affected
by the barrier.

A gene may observe its nearest neighbor. It does this by observing a signature
that is composed of the identifier of its three most active genes. These are
placed on the top of the stack, or otherwise 3 x 0.

A cell can spend energy and materials to raise a barrier in all cardinal
directions. It can also lower that barrier. A cell can spend energy and
materials to break down a neighboring cell's barrier in a cardinal direction.

The ' (QUOTE) triplet pushes the following triplet onto the stack.

+, -, *, / and % are arithmetic instructions that operate on the stack.
Triplets overflow and underflow.

> compares the two top entries on the stack. < too.

NOT substracts FFFFFF from the top of the stack.

MATCH takes the two top entries of the stack, and pushes FFFFFF to the top of
the stack if they are near enough according to a threshold, 000000 otherwise.

? executes the next instruction if the top of the stack is greater than the one
below the top of the stack.

There are no looping instructions.


Chemistry
=========

There is a simple chemistry. Various instructions can drive transformations.

H - hydrogen, never free
W - water (2 H + O)
O - oxygen (2 O)
X - carbon dioxide (1 C + 2 O)
C - carbon, never free
G - glucose (6 C + 12 H + 6 O)
S - starch (100 G) (max storage lower, cheaper to break down)
F - fat (100 G) (max storage higher, expensive to break down)
A - ATP (energy unit)
B - Barrier (for cell wall)
I - Instruction (for gene)
Y - Chemical Y, to build Barrier
Z - Chemical Z, to build Instruction

Photosyntesis: 6 X + 6 W + light => 1 G + 6 O
Respiration: 1 G + 6 O => 6 X + 6 W + 38 A
Starch genesis: 100 G + 100 A => 1 S (1 A per G stored)
Starch lysis: 1 S => 100 G
Lipogenesis: 200 G + 400 A => 1 F (2 A per G stored, twice stored compared to starch)
Lipolysis: 1 F => 200 G
Barrier genesis: 1 G + Y + 10 ATP = B
Barrier breakdown = 1 B + 20 ATP = 1 G + Y
Instruction genesis: 1 G + Z + 10 ATP = I
Instruction breakdown = 1 I + 20 ATP = 1 G + Z


Language
========

There are built-in instructions. Each is identified by a triplet.

There are also genes, which are like functions or words. A gene consists of
triplets. Each is identified by a triplet.






Reference flexibility
=====================



Genotype to Phenotype
=====================

Before execution starts, a creature's genotype is read. This is transformed
into a phenotype:


Chemistry
=========

There is a chemistry: the construction of

Stack optimization
==================

A stack costs energy by entry. A call stack all costs energy by entry. The
increase in cost is quadratic.

Stack underflow costs energy as well. Perhaps there should be a cost curve
there as well.


Fuzzy gene finding
==================

Activation uses a nearest-neighbor search for the gene to active/deactive.

Activation can search as follows:

* push a bunch of triples onto the stack.

* push the amount of triples to match on the stack.

It will find those genes that have a starting triple that matches the first
entry (with a fuzzy range). If multiple match, it will find the one that
matches most closely to the second match, until a single matches.

Drawback: we cannot cannot easily color genes anymore, or see them as within a
single space.

It's also possible to sample particular genes in the attached gene.

One a gene is found, that gene is stored in the 'gene' slot of
that gene, until another gene is matched. The 'gene' slot will
be used by all gene operators.

Ideas
=====

* Any cell-level operation involves energy input and a threshold to
  execute it. The energy input slowly decays in time.

* Cellular integrity is reduced slowly, unless energy is spent. And materials?
  Is integrity lost faster if the cell has more instructions in more genes?

* Do we want to support the implementation of higher-level operators (words)
  along with gene activation/deactivation? These higher level operators could
  be called and affect the stack of the calling gene. How to deal with
  recursion?

* Should we make arithmetic operations wrap instead of fail?

* How does conditional execution work? Two ideas:

  END which goes to the end if the stack contains 0.

  So:

  > END

  an IF construct that executes the next instruction if the stack
  contains non-null, otherwise skips.

  END sounds simplest.

* Do we want to support looping? Or is the inherent looping enough? Inherent
  looping is enough.

* Is copying done explicitly? I.e., read a gene, push the results onto
  the stack, then use the stack (or part of it) to construct a new gene?

* a clear instruction to wipe the stack clean.


* Genes have a reference to another gene, and a location on that gene.
  They can read this gene onto the stack. The location can be shifted.

* They can also write a whole new gene using the stack.


Instructions
============

There are instructions that only affect the stack.

There are instructions that affect the PC (DONE, possibly something like CALL).


There are also instructions that interact with the environment. These cost more
energy/materials.

Every gene in a cell has a handle. It's a simple index. When a gene is created,
its index is set.

[GENE_HANDLE] GENE_EXISTS

A gene can be tested for existence.

[GENE_HANDLE INDEX] GENE_READ

Read a gene.

[GENE_HANDLE CELL_HANDLE] GENE_MOVE

A gene can be moved to another cell, indicated by a connected cell handle.
This only happens if the cell handle is open.

[AMOUNT DEPTH] GENE_CREATE

A gene can be created using an amount on a stack, and a depth.


Gene replication program

0         [0]
GENE_LEN  [l]
0         [l c]     ; c: counter
DO                  ; start of loop
DUP       [l c c]
0         [l c c g] ; g: gene index
SWAP      [l c g c]
GENE_READ [l c A]  ; A: read value
SWAP      [l A c]
1         [l A c 1]
+         [l A c]  ; c++
ROT       [A c l]
SWAP      [A l c]
2DUP      [A l c l c]
>         [A l c TRUE]
LOOP      [A l c]
DROP      [A l]
1         [A l 1] ; amount, stack depth
GENE_CRE  [1] ; creates new gene, index 1
0         [1 north]
CELL_CRE  [1 0] ; cell 0 has been created
SWAP      [0 1]
GENE_MOVE [A ]    ; gene 0 into cell 0


IF ... END
IF ... ELSE ... END


[DIRECTION] CELL_CONNECT

Try to connect to another cell.

[CELL_HANDLE] CELL_EXISTS

[CELL_HANDLE] CELL_OPEN

[CELL_HANDLE] CELL_ATTACK

Attacks another cell, decreasing its strength. At some point it's
opened.



STACK MANIPULATION

From forth:

SWAP
DUP
OVER
ROT
DROP
2SWAP
2DUP
2OVER
2DROP

From factor:
removign stack elements:

drop ( x -- )

2drop ( x y -- )

3drop ( x y z -- )

nip ( x y -- y )

2nip ( x y z -- z )


Duplicating stack elements:

dup ( x -- x x )

2dup ( x y -- x y x y )

3dup ( x y z -- x y z x y z )

over ( x y -- x y x )

2over ( x y z -- x y z x y )

pick ( x y z -- x y z x )

See also here for a bigger list:

https://docs.factorcode.org/content/article-shuffle-words.html

see also Complex shuffle words:

Addressing and multi mode instructions
======================================

A triplet may be:

* push number on stack.

* execute instruction

* call gene

* do nothing (noop)

There are other instructions that manipulate genes, but
they are direct and use the stack - they ignore the bits.

Genes are identified by their top instruction (only the triple part, not the
other bits).

Control flow
============

A gene is executed, until it reaches the end. If it reaches the end and the
call stack is empty, it loops back to the beginning, otherwise it returns back
to the calling gene.

Calling can be done through call mode for a baked-in 1 instruction call,
but call can also be done from the stack, with:

<GENE_ID>
CALL

JF and JB

There is a JMPF and JMPB instruction. These jump forwards and backwards.
They are conditional jumps: they take a bool and the distance to jump
from the top of the stack. If the jump cannot be made, the jump also is
not executed.

So

TRUE
4
JMPF

jumps 4 instructions forward.

If a gene is stimulated or inhibited, this affects how fast it runs per tick.
So some functions are slower than others.


0 TRUE
1 2
2 JF (PC 3)
3
4
5 here

Processors
==========

A cell may have multiple processors. A processor runs independently
on a gene once it gets started. A cell can have multiple processors
on the same gene.

<GENE_ID>
PROC_START

starts a processor on that gene.

<GENE_ID>
PROC_END kills all processors on a gene.

PROC_DONE kills the current processor.

There is a maximum amount of processors per gene, but
since running a processor takes ATP, running out of ATP also is trouble.

Death
=====

A cell can die or be killed.

A cell dies if it's out of ATP.

Should we accept higher values on the stack?
============================================

We currently interpret them as noop if encountered
as instructions. So that should be okay.


A compilation process
=====================

Before we start executing instructions, we compile
it down to definite instruction calls and other
actions. ala

enum CompiledInstruction {
    Instruction(instruction),
    Number(number: u32)
}

We still have fuzzy lookup for calls. We can do the compilation as soon as a
gene is created.