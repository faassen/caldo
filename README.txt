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


