# Lazy Sequences

This repository aims to provide an expressive yet practically useful set of abstractions for lazily working with potentially infinitely large sequences. It takes the form of a library for the rust programming language, but the concepts should be applicable to any strictly evaluated programming language.

In the rust world, the APIs generalize a number of traits which are already in common use, and they put them into a consistent framework as opposed to the current selection of multiple, individual ad-hoc designs. Important traits that are subsumed include [`core::iter::Iterator`](https://doc.rust-lang.org/core/iter/trait.Iterator.html), [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html), [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html), [`futures::Stream`](https://docs.rs/futures/0.3.7/futures/stream/trait.Stream.html), [`futures::Sink`](https://docs.rs/futures/0.3.7/futures/sink/trait.Sink.html), [`bytes::Buf`](https://docs.rs/bytes/0.6.0/bytes/trait.Buf.html) and [`bytes::BufMut`](https://docs.rs/bytes/0.6.0/bytes/trait.BufMut.html).

## The Underlying Model

We aim to design APIs for working with homogeneous sequences containing items of some type `Item`. Some examples include the sequence consisting of the integers `3`, `7`, `4`, `4` (a finite sequence of length four), all integers greater than `0` (an infinite sequence with a start), all integers less than `0` (an infinite sequence with an end), and all integers (an infinite sequence with neither a start nor an end). While it is possible to fit a finite sequence of items into memory (e.g. as a `Box<[Item]>`), the same cannot be done for sequences of infinite length.

APIs working with infinite sequences instead operate locally somewhere within the sequence, and then provide some mechanism for moving the position at which one operates. The `Interator` trait's `next` method for example reads the item at the current position and also advances the position by one. We can easily imagine other traits which provide different functionality, such as writing items to the sequence, or moving the position backwards rather than forwards. This is the approach taken in this repository: first we define the conceptual layout of an infinite sequence with a cursor, and then we provide traits that describe different ways of manipulating the sequence and/or the cursor.

The model is effectively a fine-grained view of a stationary tape and mobile head of a [Turing machine](https://en.wikipedia.org/wiki/Turing_machine), except that the head rests between cells rather than on cells. The infinite tape for a sequence holding items of type `Item` (conceptually) consists of values of type [`Option<Item>`](https://doc.rust-lang.org/std/option/enum.Option.html). A `None` represents a blank symbol, the `Some` values hold non-blank characters of the tape alphabet (with each possible value of type `Item` corresponding to a character). At any point in time there is exactly one designated *current position* in between exactly two of the tape's cells.

The diagram below shows a tape consisting of infinitely many `None`s, followed by `Some(3)`, `Some(7)`, `Some(4)`, `Some(4)`, followed by infinitely many `None`s. The current position is in between the cells holding the `3` and the `7`.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
               XXX
```

All traits for manipulating the conceptual tape and head all extend the `SequenceManipulator` base trait:

```rust
trait SequenceManipulator {
    type Item;
    type In;
}
```

The `Item` the type should be straightforward, it is the type of items being held on the tape. `In` is short for *internal* and allows sequence manipulators to uphold invariant for the tape and the head. All of the methods of subtraits which allow manipulating tape or head return a [`Result<Something, Self::In>`](https://doc.rust-lang.org/std/result/enum.Result.html). Rather than performing the desired manipulation, any such method can instead return an error, with the contained value of type `Self::In` detailing why the operation could not be performed. In general, a method must either fully perform the desired manipulation and return an `Ok`, or it must leave tape and head completely unchanged and return an `Err`.

As an example, an iterator-like sequence manipulator could use `type In = ()`, and retrieving the next item would either return `Ok(some_item)` if the tape contained a `Some(some_item)` and move the head one position to the right, or it would return an `Err(())` and leave the tape unchanged if it contained a `None`.

Aside from enforcing invariants, this can also be used to represent fallible, effectfull manipulators (e.g. writing to a TCP stream). Code that interacts with a manipulator must in general assume that any method calls performed after a method returned an error exhibit unspecified behavior. Implementers of the manipulator interfaces are of course free to commit to unspecified semantics after an internal state change.

We now present a number of traits that provide different modes of interaction with the tape and the head. If a type implements multiple of these, they all conceptually interact with a single tape and head. Three traits for reading, writing and moving the head could for example be combined to express the abstraction of a seekable file.

Issues such as buffering, size hints and other optimizations, closing or asynchronicity are discussed after the traits have been introduced in the simplest possible setting.

## Manipulators

Due to rust ownership and borrowing semantics, the operations at the head need to be more varied than the simple read/write/move operations of a Turing machine.

### Skippers

```rust
trait Skipper {
    fn skip(&mut self) -> Result<(), Self::In>;
    fn skip_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.skip()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`skip` moves the current position by one to the right. `skip_many1` performs up to `amount` many moves and returns how many have been performed.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
               XXX

  self.skip() ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                   XXX
```

All manipulators also come with a version that works in the reverse direction, in this case:

```rust
trait RevSkipper {
    fn rev_skip(&mut self) -> Result<(), Self::In>;
    fn rev_skip_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.rev_skip()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
               XXX

  self.rev_skip() ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
           XXX
```

### Taker

```rust
trait Taker {
    fn take(&mut self) -> Result<Self::Item, Self::In>;
    fn take_ignore(&mut self) -> Result<(), Self::In> {
        let _ = self.take()?;
        Ok(())
    }
    fn take_ignore_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.drop()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`take` must result in an `Err` if the value to the right of the current position is a `None`. Otherwise, it moves the value out of the `Some` and returns the value, leaving a `None`. The current position is then moved by one to the right.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
               XXX

  self.take() ---> Ok(7)

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 |   | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                   XXX
```

`take_ignore` conceptually does the same but drops the contained value rather than returning it. `take_ignore_many1` performs up to `amount` many `take_ignore`s and returns how many have been performed.

### Giver

```rust
trait Giver {
    fn give(&mut self, item: Self::Item) -> Result<(), Self::In>;
    fn give_default(&mut self) -> Result<(), Self::In> where Self::Item: Default {
        self.give(Self::Item::default())
    }
    fn give_default_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> where Self::Item: Default {
        let _ = self.give_default()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`give` must result in an `Err` if the value to the right of the current position is a `Some`. Otherwise, it moves the input item into the `None`. The current position is then moved by one to the right.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 |   |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                       XXX

  self.give(4) ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                           XXX
```

`give_default` does the same but gives a default value of an input. `give_default_many1` performs up to `amount` many `give_default` and returns how many have been performed.

### Swapper

```rust
trait Swapper {
    fn swap(&mut self, item: Self::Item) -> Result<Self::Item, Self::In>;

    fn swap_default(&mut self) -> Result<Self::Item, Self::In> where Self::Item: Default {
        self.swap(Self::Item::default())
    }

    fn swap_ignore(&mut self, item: Self::Item) -> Result<(), Self::In> {
        let _ = self.swap(item)?;
        Ok(())
    }

    fn swap_default_ignore(&mut self) -> Result<(), Self::In> where Self::Item: Default {
        self.swap_ignore(Self::Item::default())
    }

    fn swap_default_ignore_many1(&mut self, amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> where Self::Item: Default {
        let _ = self.swap_default_ignore()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}
```

`swap` must result in an `Err` if the value to the right of the current position is a `None`. Otherwise, it moves the input item into the `Some` and returns the previously contained item. The current position is then moved by one to the right.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                   XXX

  self.swap(5) ---> Ok(4)

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 5 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                       XXX
```

`swap_ignore` drops the previously contained value, `swap_default` swap in a default value, and `swap_default_ignore` does both. `swap_default_ignore_many1` performs up to `amount` many `swap_default_ignore` and returns how many have been performed.

### Updater

```rust
trait Updater {
    fn update<F: FnOnce(Self::Item) -> Self::Item>(&mut self, f: F) -> Result<(), Self::In>;
}
```

`update` must result in an `Err` if the value to the right of the current position is a `None`. Otherwise, it passes the value in the `Some` into the function and places the result back. The current position is then moved by one to the right.

```ascii
  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 4 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                   XXX

  self.swap(|n| n + 1) ---> Ok(())

  --+---+---+---+---+---+---+---+---+--
... |   |   | 3 | 7 | 5 | 4 |   |   | ...
  --+---+---+---+---+---+---+---+---+--
                       XXX
```
