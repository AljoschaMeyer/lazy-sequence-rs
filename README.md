# Lazy Sequences

**status: WIP, some traits still missing, not ready for publishing on crates.io, but everything in the readme should be fairly stable**

This repository aims to provide an expressive yet practically useful set of abstractions for lazily working with potentially infinitely large sequences. It takes the form of a library for the rust programming language, but the concepts should be applicable to any strictly evaluated programming language.

In the rust world, the APIs generalize a number of traits which are already in common use, and they put them into a consistent framework as opposed to the current selection of multiple, individual ad-hoc designs. Important traits that are subsumed include [`core::iter::Iterator`](https://doc.rust-lang.org/core/iter/trait.Iterator.html), [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html), [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html), [`futures::Stream`](https://docs.rs/futures/0.3.7/futures/stream/trait.Stream.html), [`futures::Sink`](https://docs.rs/futures/0.3.7/futures/sink/trait.Sink.html), [`bytes::Buf`](https://docs.rs/bytes/0.6.0/bytes/trait.Buf.html) and [`bytes::BufMut`](https://docs.rs/bytes/0.6.0/bytes/trait.BufMut.html).

## The Underlying Model

We aim to design APIs for working with homogeneous sequences containing items of some type `Item`. Some examples include the sequence consisting of the integers `3`, `7`, `4`, `4` (a finite sequence of length four), all integers greater than `0` (an infinite sequence with a start), all integers less than `0` (an infinite sequence with an end), and all integers (an infinite sequence with neither a start nor an end). While it is possible to fit a finite sequence of items into memory (e.g. as a `Box<[Item]>`), the same cannot be done for sequences of infinite length.

APIs working with infinite sequences instead operate locally somewhere within the sequence, and then provide some mechanism for moving the position at which one operates. The `Iterator` trait's `next` method for example reads the item at the current position and also advances the position by one. We can easily imagine other traits which provide different functionality, such as writing items to the sequence, or moving the position backwards rather than forwards. This is the approach taken in this repository: first we define the conceptual layout of an infinite sequence with a cursor, and then we provide traits that describe different ways of manipulating the sequence and/or the cursor.

The model is effectively a fine-grained view of the stationary tape and mobile head of a [Turing machine](https://en.wikipedia.org/wiki/Turing_machine). The infinite tape for a sequence holding items of type `Item` (conceptually) consists of values of type [`Option<Item>`](https://doc.rust-lang.org/std/option/enum.Option.html). A `None` represents a blank symbol, the `Some` values hold non-blank characters of the tape alphabet (with each possible value of type `Item` corresponding to a character). At any point in time there is exactly one designated *current cell*.

The diagram below shows a tape consisting of infinitely many `None`s, followed by `Some(3)`, `Some(7)`, `Some(4)`, `Some(4)`, followed by infinitely many `None`s. The current position is the cell holding the `7`.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

All traits for manipulating the conceptual tape and current cell extend the `SequenceManipulator` base trait:

```rust
trait SequenceManipulator {
    type Item;
    type In;
}
```

The `Item` the type should be straightforward, it is the type of items being held on the tape. `In` is short for *internal* and allows sequence manipulators to uphold invariant for the tape and the head. All of the methods of subtraits which allow manipulating tape or head return a [`Result<Something, Self::In>`](https://doc.rust-lang.org/std/result/enum.Result.html). Rather than performing the desired manipulation, any such method can instead return an error, with the contained value of type `Self::In` detailing why the operation could not be performed. In general, a method must either fully perform the desired manipulation and return an `Ok`, or it must leave tape and current cell completely unchanged and return an `Err`.

As an example, an iterator-like sequence manipulator could use `type In = ()`, and retrieving the next item would either return `Ok(some_item)` if the current cell contained a `Some(some_item)` and move the head one position to the right, or it would return an `Err(())` and leave the tape unchanged if it contained a `None`.

Aside from enforcing invariants, this can also be used to represent fallible, effectfull manipulators (e.g. writing to a TCP stream). Code that interacts with a manipulator must in general assume that any method calls performed after a method returned an error exhibit unspecified behavior. Implementers of the manipulator interfaces are of course free to commit to unspecified semantics after an internal state change.

We now present a number of traits that provide different modes of interaction with the tape and the head. If a type implements multiple of these, they all conceptually interact with a single tape and current cell. Three traits for reading, writing and moving the current cell could for example be combined to express the abstraction of a seekable file.

Issues such as buffering, closing, size hints, and other optimizations are discussed after the traits have been introduced in the simplest possible setting. We do not discuss asynchronous interfaces, the idea being that simply making all methods `async` does the trick.

### Moving the Current Cell

The `Next` and `Prev` traits allow moving the current cell relative to its previous position. It is not possible to set the new current cell by an absolute value because sequences can be infinite and addressing cells on an infinite tape is tricky. The two directions of movement are split up into separate traits because many natural concepts only require moving into a single direction - for example it is not possible to generate for an iterator to go backwards.

```rust
trait Next: SequenceManipulator {
    fn next(&mut self) -> Result<(), Self::In>;
    fn next_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.next()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`next` moves the current position by one to the right. `next_many1` performs up to `amount` many `next` and returns how many have been performed.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.next() ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                     XXX
```

```rust
trait Prev: SequenceManipulator {
    fn prev(&mut self) -> Result<(), Self::In>;
    fn prev_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.prev()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`prev` moves the current position by one to the left. `prev_many1` performs up to `amount` many `prev` and returns how many have been performed.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.prev() ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
             XXX
```

### Changing Ownership

Writing to or reading from the sequence involves transfer of ownership of the involved values. `Read` describes how values can be moved out of a cell on the tape:

```rust
trait Read: SequenceManipulator {
    fn read(&mut self) -> Result<Self::Item, Self::In>;
}
```

`read` must result in an `Err` if the value at the current position is a `None`. Otherwise, it moves the value out of the `Some` and returns the value, leaving a `None`.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.read() ---> Ok(7)

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 |   | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

There is a quite natural dual to the `Read` trait: `Write` for inserting items into a sequence.

```rust
trait Write: SequenceManipulator {
    fn write(&mut self, item: Self::Item) -> Result<(), Self::In>;
}
```

`write` must result in an `Err` if the value at the current position is a `Some`. Otherwise, it moves the input item into the `None`.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 |   |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                         XXX

  self.write(4) ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                         XXX
```

### Borrowing Input

Rather than moving items into the tape, it is sometimes sufficient to pass a reference to an item as an input to the sequence manipulator, which then uses the reference to compute the value to insert into the sequence.

`WriteRefInLong` receives input through a reference which outlives the manipulator itself:

```rust
trait WriteRefInLong: SequenceManipulator {
    fn write_ref_in_long<'s, 'i: 's>(&'s mut self, item: &'i Self::Item) -> Result<(), Self::In>;
}
```

`write_ref_in_long` must result in an `Err` if the value at the current position is a `Some`. Otherwise, it computes some value from the input reference and moves that value into the `None`.

`WriteRefIn` provides `write_ref_in` which works the same way except that the lifetime of the input reference is limited.

```rust
trait WriteRefIn: WriteRefInLong {
    fn write_ref_in(&mut self, item: &Self::Item) -> Result<(), Self::In>;
}
```

Just like an immutable input reference can be used for writing, a mutable input reference can be used for reading.

```rust
trait ReadRefInLong: SequenceManipulator {
    fn read_ref_in_long<'s, 'i: 's>(&'s mut self, item: &'i mut Self::Item) -> Result<(), Self::In>;
}
```

`read_ref_in_long` must result in an `Err` if the value at the current position is a `None`. Otherwise, it moves the value out of the `Some` and uses it to somehow mutate the input reference.

`ReadRefIn` provides `read_ref_in` which works the same way except that the lifetime of the input reference is limited.

```rust
trait ReadRefIn: ReadRefInLong {
    fn read_ref_in(&mut self, item: &mut Self::Item) -> Result<(), Self::In>;
}
```

### Lending Output

The next pair of traits is about handing out references to values on the tape rather than moving values out of it.

```rust
trait WriteRefOut: SequenceManipulator {
    fn write_ref_out(&mut self) -> Result<*mut Self::Item, Self::In>;
}
```

`write_ref_out` must result in an `Err` if the value at the current position is a `None`. Otherwise, it returns a mutable pointer to the current item. This pointer must be treated like a reference whose lifetime ends exactly when the current position moves. This requirement is dynamic so it cannot be tracked by an actual lifetime.

There are however manipulators where the returned reference is known to outlive the manipulator itself, for these manipulators that a lifetime can be tracked in the type system.

```rust
trait WriteRefOutLong: WriteRefOut {
    fn write_ref_out_long(&mut self) -> Result<&mut Self::Item, Self::In>;
}
```

Reading via a reference is accomplished by the `ReadRefOut` trait:

```rust
trait ReadRefOut: SequenceManipulator {
    fn read_ref_out(&mut self) -> Result<*const Self::Item, Self::In>;
}
```

`read_ref_out` must result in an `Err` if the value at the current position is a `None`. Otherwise, it returns an immutable pointer to the current item. This pointer must be treated like a reference whose lifetime ends exactly when the current position moves.

For cases where the references known to stay valid through the full lifetime of the manipulator, there is the annalogous `ReadRefOutLong` trait:

```rust
trait ReadRefOutLong: ReadRefOut {
    fn read_ref_out_long(&mut self) -> Result<&Self::Item, Self::In>;
}
```

### Passing Ownership Through Pointers

While idiomatic rust does ownership transfer by simply passing around values, in other languages it is common to pass around pointers from which the value is being read from or written to. The following traits allow for this mode of interaction with the tape. Idiomatic code should prefer the `Read` and `Write` traits, the pointer-based APIs are mostly provided because unlike `Read` and `Write` they generalize nicely to transfer of multiple value simultaneously.

```rust
trait WriteIn: SequenceManipulator {
    unsafe fn write_in(&mut self, item: *const Self::Item) -> Result<(), Self::In>;
}
```

`write_in` must result in an `Err` if the value at the current position is a `Some`. Otherwise, exactly one call to [`core::ptr::read`](https://doc.rust-lang.org/std/ptr/fn.read.html) is performed on the item pointer and the resulting value is written to the tape. It is the caller's responsibility to ensure that `core::ptr::read` does not lead to undefined behavior. An implementation of `write_in` may not result in undefined behavior if the item pointer is properly readable.

```rust
trait ReadIn: SequenceManipulator {
    unsafe fn read_in(&mut self, item: *mut Self::Item) -> Result<(), Self::In>;
}
```

`read_in` must result in an `Err` if the value at the current position is a `None`. Otherwise, the value on the tape is moved out of the tape and used for exactly one call to [`core::ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) on the item pointer. It is the caller's responsibility to ensure that `core::ptr::writer` does not lead to undefined behavior. An implementation of `read_in` may not result in undefined behavior if the item pointer is properly writeable.

```rust
trait WriteOut: SequenceManipulator {
    unsafe fn write_out(&mut self) -> Result<*mut Self::Item, Self::In>;
}
```

`write_out` must result in an `Err` if the value at the current position is a `Some`. Otherwise, it returns a mutable pointer to the current position, which may or may not point to uninitialized memory. The caller must call [`core::ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) on this pointer at least once - violating this contract may result in undefined behavior. The pointer may not be used after the current position as changed, doing so may result in undefined behavior. An implementation of `write_out` may not result in undefined behavior if those two rules are followed by the calling code.

```rust
trait ReadOut: SequenceManipulator {
    unsafe fn read_out(&mut self) -> Result<*const Self::Item, Self::In>;
}
```

`read_out` must result in an `Err` if the value at the current position is a `None`. Otherwise, it returns an immutable pointer to the current item. The caller should call [`core::ptr::read`](https://doc.rust-lang.org/std/ptr/fn.read.html) on this pointer at least once. The pointer may not be used after the current position as changed, doing so may result in undefined behavior. An implementation of `write_out` may not result in undefined behavior in any other case.

## Buffering

It is a common pattern for sequence manipulators to employ *buffering* to improve efficiency. This is due to the fact that many real-world implementations of the conceptual tape can be rather expensive to read from or write to, but reading or writing multiple, adjacent cells is not a lot more expensive than reading or writing single cells.

A buffered writer thus writes data to a small, internal buffer that is cheap to write to. Whenever that buffer becomes full, lots of its contents are written simultaneously to the actual tape. This can happen completely transparently. The following diagrams show a writer with a buffer of two cells.

```ascii
            +---+---+
            |   |   | Buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   |   |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
             XXX

  self.write(4) ---> Ok(())
  self.next() ---> Ok(())

            +---+---+
            | 4 |   | Buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   |   |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.write(7) ---> Ok(())
  self.next() ---> Ok(())

            +---+---+
            | 4 | 7 | Buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   |   |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                     XXX

  self.write(5) ---> Ok(())
  self.next() ---> Ok(())

                    +---+---+
                    | 5 |   | Buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   | 4 | 7 |   |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                         XXX
```

In addition to this transparent buffer handling, it is sometimes necessary to be able to actively flush a buffer even though there is still space, most often because another piece of code is observing the tape and should receive the new information immediately. This functionality is provided by via the `FlushPrev` and `FlushNext` traits.

```rust
trait FlushPrev: SequenceManipulator {
    fn flush_prev(&self) -> Result<(), Self::In>;
}
```

`flush_prev` flushes the positions in the buffer that are to the left of the current position and the current position itself.

```ascii
            +---+---+---+
            | 4 | 7 | 5 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   |   |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.flush_prev() ---> Ok(())

            +---+---+---+
            |   |   | 5 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   | 4 | 7 |   |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

```rust
trait FlushNext: SequenceManipulator {
    fn flush_next(&self) -> Result<(), Self::In>;
}
```

`flush_next` flushes the positions in the buffer that are to the right of the current position and the current position itself.

```ascii
            +---+---+---+
            | 4 | 7 | 5 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   |   |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.flush_next() ---> Ok(())

            +---+---+---+
            | 4 |   |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   | 7 | 5 |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

A buffered *reader* reads data from an internal buffer and optimistically moves lots of data from the tape to the buffer on each "buffer miss".

```ascii
            +---+---+
            |   |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   | 3 | 4 | 7 | 5 | 2 |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
             XXX

  self.read() ---> Ok(4)
  self.next() ---> Ok(())

            +---+---+
            |   | 7 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   | 3 |   |   | 5 | 2 |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.read() ---> Ok(7)
  self.next() ---> Ok(())

            +---+---+
            |   |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   | 3 |   |   | 5 | 2 |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                     XXX

  self.read() ---> Ok(5)
  self.next() ---> Ok(())

                    +---+---+
                    |   | 2 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   | 3 |   |   |   |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                         XXX
```

In addition to this transparent buffer handling, it is sometimes necessary to be able to actively slurp data into a buffer even though there is still data in the buffer, most often because another piece of code has modified the tape and the reader would otherwise work with stale data. This functionality is provided by via the `SlurpPrev` and `SlurpNext` traits.

```rust
trait SlurpPrev: SequenceManipulator {
    fn slurp_prev(&self) -> Result<(), Self::In>;
}
```

`slurp_prev` slurps the positions in the buffer that are to the left of the current position and the current position itself.

```ascii
            +---+---+---+
            |   |   |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   | 4 | 7 | 5 |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.slurp_prev() ---> Ok(())

            +---+---+---+
            | 4 | 7 |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   |   |   | 5 |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

```rust
trait SlurpNext: SequenceManipulator {
    fn slurp_next(&self) -> Result<(), Self::In>;
}
```

`slurp_next` slurps the positions in the buffer that are to the right of the current position and the current position itself.

```ascii
            +---+---+---+
            |   |   |   | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   | 4 | 7 | 5 |   |   |   | ... tape
  --+---+---+---+---+---+---+---+---+--
                 XXX

  self.slurp_next() ---> Ok(())

            +---+---+---+
            |   | 7 | 5 | buffer
  --+---+---+---+---+---+---+---+---+--
... |   |   | 4 |   |   |   |   |   | ... Tape
  --+---+---+---+---+---+---+---+---+--
                 XXX
```

### Transferring Multiple Items

The traits for reading or writing via pointers/references can easily be extended to transfer multiple items at a time by referencing a nonempty slice (a [`Loaf`](https://crates.io/crates/loaf)) rather than individual items. Traits taking the pointer/reference as input return how many items have been processed. In those cases, it is the sequence manipulator itself that decides how many items are used. The traits returning pointers/references leave this decision up to the caller.

When returning a reference, there is again a distinction between references that outlive the sequence manipulator, and those that only stay valid for a short, dynamically determined time. The window of valid indexes within the output loaf starts out as the whole loaf. Moving the current position to the right also moves the window of valid indexes to the right, moving the current position to the left moves the window to the left. A index becomes invalid if it ever leaves that window - reentering the window does not make the position valid again. Flushing a position also invalidates it.

```rust
trait WriteRefInLongMany1: WriteRefInLong {
    fn write_ref_in_long_many1<'s, 'i: 's>(&'s mut self, items: &'i Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait WriteRefInMany1: WriteRefInLongMany1 {
    fn write_ref_in_many1(&mut self, item: &Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait ReadRefInLongMany1: ReadRefInLong {
    fn read_ref_in_long_many1<'s, 'i: 's>(&'s mut self, item: &'i mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait ReadRefInMany1: ReadRefInLongMany1 {
    fn read_ref_in_many1(&mut self, item: &mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait WriteInMany1: WriteIn {
    unsafe fn write_in_many1(&mut self, item: *const Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait ReadInMany1: ReadIn {
    unsafe fn read_in_many1(&mut self, item: *mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}
```

```rust
trait WriteRefOutMany1: WriteRefOut {
    fn write_ref_out_many1(&mut self) -> Result<*mut Loaf<Self::Item>, Self::In>;
}
```

```rust
trait WriteRefOutLongMany1: WriteRefOutMany1 {
    fn write_ref_out_long_many1(&mut self) -> Result<&mut Loaf<Self::Item>, Self::In>;
}
```

```rust
trait ReadRefOutMany1: ReadRefOut {
    fn read_ref_out_many1(&mut self) -> Result<*const Loaf<Self::Item>, Self::In>;
}
```

```rust
trait ReadRefOutLongMany1: ReadRefOutMany1 {
    fn read_ref_out_long_many1(&mut self) -> Result<&Loaf<Self::Item>, Self::In>;
}
```

```rust
trait WriteOutMany1: WriteOut {
    unsafe fn write_out_many1(&mut self) -> Result<*mut Loaf<Self::Item>, Self::In>;
}
```

```rust
trait ReadOutMany1: ReadOut {
    unsafe fn read_out_many1(&mut self) -> Result<*const Loaf<Self::Item>, Self::In>;
}
```

## Stopping

Notifying a sequence manipulator that no more reads or writes will be performed can often allow it to free resources.

```rust
trait StopRead: SequenceManipulator {
    type StopR;
    fn stop_read(&self, reason: Self::StopR) -> Result<(), Self::In>;
}
```

`stop_read` tells the sequence manipulator that no more methods will be called that read from the tape/buffer. Failure to hold this promise can result in unspecified behavior. `StopR` is the type of an argument that can provide additional information about why reading is being stopped.

The analogous trait for stopping to write is `StopWrite`:

```rust
trait StopWrite: SequenceManipulator {
    type StopW;
    fn stop_write(&self, reason: Self::StopW) -> Result<(), Self::In>;
}
```

TODO size hints, swap, overwrite, elastic tape (insert and delete)
