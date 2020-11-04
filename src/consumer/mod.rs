use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr;

use loaf::Loaf;

trait Consumer {
    type Item;
    type Ex;
    type In;
    fn consume(&mut self, item: Self::Item) -> Result<(), Self::In>;
    fn flush(&mut self) -> Result<(), Self::In>;
    fn close(&mut self, ex: Self::Ex) -> Result<(), Self::In>;

    fn consume_flush(&mut self, item: Self::Item) -> Result<(), Self::In> {
        self.consume(item)?;
        self.flush()
    }
}

trait ConsumerFrom: Consumer {
    unsafe fn consume_from(&mut self, from: *const Self::Item) -> Result<(), Self::In> {
        self.consume(ptr::read(from))
    }

    unsafe fn consume_from_flush(&mut self, from: *const Self::Item) -> Result<(), Self::In> {
        self.consume_from(from)?;
        self.flush()
    }
}

trait ConsumerFromMany1: ConsumerFrom {
    unsafe fn consume_from_many1(&mut self, from: *const Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;

    unsafe fn consume_from_many1_flush(&mut self, from: *const Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In> {
        let consumed = self.consume_from_many1(from)?;
        self.flush()?;
        Ok(consumed)
    }
}

trait ConsumerTo: Consumer {
    fn consume_to(&mut self) -> Option<*mut MaybeUninit<Self::Item>>;
    unsafe fn do_consume_to(&mut self) -> Result<(), Self::In>;

    unsafe fn do_consume_to_flush(&mut self) -> Result<(), Self::In> {
        self.do_consume_to()?;
        self.flush()
    }
}

trait ConsumerToMany1: ConsumerTo {
    fn consume_to_many1(&mut self, max: NonZeroUsize) -> Option<*mut Loaf<MaybeUninit<Self::Item>>>;
    unsafe fn do_consume_to_many1(&mut self) -> Result<NonZeroUsize, Self::In>;

    unsafe fn do_consume_to_many1_flush(&mut self) -> Result<NonZeroUsize, Self::In> {
        let consumed = self.do_consume_to_many1()?;
        self.flush()?;
        Ok(consumed)
    }
}
