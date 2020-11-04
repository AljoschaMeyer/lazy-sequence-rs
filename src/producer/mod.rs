use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr;

use loaf::Loaf;

/// The basic interface for reading items from a lazy sequence.
///
/// Conceptually, a `Producer` maintains a cursor into a sequence. Calling `produce` advances that
/// cursor by one step and yields the sequence item over which it stepped.
///
/// Any call of a producer method can result in a `Self::In`, signaling that an *in*ternal state
/// change occurred rather than performing the method. Reasons for this include having reached the
/// end of the sequence, or perhaps an effect for computation failing. In any case, the conceptual
/// cursor does not move on an internal state change - any method triggering an internal state
/// change has no other effects.
///
/// Any further method calls after an internal state change have unspecified behavior
/// and should thus not be performed. This also applies to the methods of subtraits. Individual
/// implementations are of course free to specify the behavior of method calls after an internal
/// state change.
///
/// The code working with a producer can also externally trigger a state change by calling `stop`.
/// Any further calls to methods of the producer result in unspecified behavior - calling `stop` is
/// essentially a promise that no further actions will be performed.
///
/// Producers can be buffered, aiming to obtain new items to produce rarely but in large amounts.
/// A producer can be forced to immediately obtain as many new items as possible at that point by
/// calling `slurp`. It is never required to slurp explicitly in order to prevent running out of
/// items, `produce` implicitly slurps when necessary.
trait Producer {
    /// The type of items that are produced.
    type Item;

    /// The information passed to the producer when calling `stop`. Short for *ex*ternal state
    /// change.
    type Ex;

    /// The information handed out by the producer after an *in*ternal state change. Internal state
    /// can happen during any method call. Methods are never performed partially: in case of an
    /// internal state change, it is as if the original method call never happened.
    type In;

    /// Either successfully yield the next item from the sequence, advancing the conceptual cursor
    /// by one, or signal an internal state change.
    fn produce(&mut self) -> Result<Self::Item, Self::In>;

    /// Immediately fill the internal buffer of the producer as much as possible without blocking.
    fn slurp(&mut self) -> Result<(), Self::In>;

    /// Signals to the producer that no more methods will be called.
    fn stop(&mut self, ex: Self::Ex) -> Result<(), Self::In>;

    /// Slurps and then immediately produces an item.
    ///
    /// Implementations of this function can bypass the internal buffer, making it more efficient
    /// than calling `slurp` and `produce` separately.
    fn slurp_produce(&mut self) -> Result<Self::Item, Self::In> {
        self.slurp()?;
        self.produce()
    }
}

// trait ProducerTo: Producer {
//     unsafe fn produce_to(&mut self, to: *mut Loaf<MaybeUninit<Self::Item>>) -> Result<NonZeroUsize, Self::In>;
//
//     unsafe fn slurp_produce_to(&mut self, to: *mut Loaf<MaybeUninit<Self::Item>>) -> Result<NonZeroUsize, Self::In> {
//         self.slurp()?;
//         self.produce_to(to)
//     }
//
//     unsafe fn produce_to_1(&mut self, to: *mut MaybeUninit<Self::Item>) -> Result<(), Self::In> {
//         Ok(ptr::write(to, MaybeUninit::new(self.produce()?)))
//     }
//
//     unsafe fn slurp_produce_to_1(&mut self, to: *mut MaybeUninit<Self::Item>) -> Result<(), Self::In> {
//         self.slurp()?;
//         self.produce_to_1(to)
//     }
// }
//
// trait ProducerFrom: Producer {
//     fn produce_from(&mut self) -> Result<*const Loaf<Self::Item>, Self::In>;
//     unsafe fn do_produce_from(&mut self, amount: NonZeroUsize) -> ();
//
//     // unsafe fn slurp_do_produce_from(&mut self, amount: NonZeroUsize) -> Result<(), Self::In> {
//     //     self.slurp()?;
//     //     Ok(self.do_produce_from(amount))
//     // }
//
//     // fn produce_from_1(&mut self) -> Result<*const Item, Self::In> {
//     //
//     // }
//
//     // unsafe fn do_produce_from_1(&mut self) -> () {
//     //
//     // }
//
//     // unsafe fn slurp_do_produce_from_1(&mut self) -> Result<(), Self::In> {
//     //     self.slurp()?;
//     //     Ok(self.do_produce_from_1())
//     // }
// }

impl<T> Producer for Option<T> {
    type Item = T;

    /// Returned when attempting to `produce` a value from a `None`.
    ///
    /// This implementation allows calling further methods after an internal state change.
    type In = ();

    /// No information is transmitted when calling `stop`.
    type Ex = ();

    /// Moves the value out of the option if it contains one, or does an internal state change
    /// in case of a `None`.
    fn produce(&mut self) -> Result<T, Self::In> {
        self.take().ok_or(())
    }

    /// No-op, always returns `Ok(())`.
    fn slurp(&mut self) -> Result<(), Self::In> {
        Ok(())
    }

    /// No-op, always returns `Ok(())`.
    fn stop(&mut self, _ex: Self::Ex) -> Result<(), Self::In> {
        Ok(())
    }
}
