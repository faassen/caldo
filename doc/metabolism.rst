Metabolism
==========

We have catalyst ports identified with chemical reactions. If the cell solves a
problem in a port, the chemical reaction takes place.

Maintaining structures in the cell is also a catalyst port -- it is like a
chemical reaction (for instance, to maintain the cell wall).

Moving the cell is also a catalyst port -- it turns a chemical into
movement. This loses the energy.

A cell can also connect to another cell, and then transfer chemicals
and genes. A cell can create a new cell it is connected to.

How does a connection take place?

A cell can set up a communication channel with another one.

  cell 1   cell 2
  input <- output
  output -> input
  gt: off      on
  ct: on      off

This communication channel behaves like a port inside. The cell can send it
values and receive values. The cell can decide based on its interaction that it
wants to open itself to gene transfers, and open itself to chemical transfers.
It can also transfer a gene through the port with a special instruction.
Chemicals are transferred through a port with a special instruction too.

How would a sensor port work? Its input and output mechanism appears to be
reverted: the cell directs the sensing to the output port first, and the input
port then delivers the information. Once the information is fully read, the
output port is ready to receive new items. But is there a benefit to this
compared to simply having instructions to do this? Not really, so let's not go
here. Or perhaps we should as communication ports are basically random too?



Metabolism and ports.

An organelle is something that is:

* lookup addressable.

* has a unique organelle id per cell

Organelles can do different things:

* Observe/talk to the outside world.

* Do metabolism.

* Ingest/expel chemicals to/from the outside world.

* Create new cell.

* Create a new gene?

* Transfer/admit genes from another, connected cell.

Metabolism organelles
---------------------

A gene needs to get items from its queue and then send items back.

How does this work -- what if the queue is very full? Or is the queue
never going to be added to until an answer is given? I think that can work.

If a more correct answer is given, more metabolism takes place.

How much of the "reaction" takes place depends on how much the cell
wants to do, with some gradual limits perhaps.

Other actions
-------------

Each action has a lookup id.

Other organelles can also be addressed to cause chemicals to be ingested
or expelled.

Connection
----------

An organelle may cause a connection to another. Or perhaps connections
are their own thing.


Possible operations
-------------------

Basic mathematical operators

Basic equality operators

Basic boolean operators

Combinations of such - an exponential equation for instance.

Or text prediction.

The general mechanism
---------------------

The system generates an input sequence and an output sequence.

It maintains a pointer into the input sequence and a pointer into the
output sequence.

When being read from the input queue, the pointer is moved. If the input
pointer is beyond the end of the input, no more input is forthcoming.

When values are pushed to the output queue, the output pointer is moved. If the
output pointer is the same length as the expected output, the input and output
are compared and a reward (the metabolism) is given.

A new input and expected output are then generated and the pointers are
both reset to 0. This generation can also happen "on demand" when input
is requested when the input pointer is 0 or output is pushed when the
output pointer is still 0.

Output may be pushed before the input is fully read. If this happens, the
problem is reset and the input queue is cleared.