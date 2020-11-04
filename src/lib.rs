#![no_std]

use core::num::NonZeroUsize;

use loaf::Loaf;

trait SequenceManipulator {
    type Item;
    type In;
}

trait Next: SequenceManipulator {
    fn next(&mut self) -> Result<(), Self::In>;
    fn next_many1(&mut self, _amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.next()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

trait Prev: SequenceManipulator {
    fn prev(&mut self) -> Result<(), Self::In>;
    fn prev_many1(&mut self, _amount: NonZeroUsize) -> Result<NonZeroUsize, Self::In> {
        let _ = self.prev()?;
        Ok(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

trait Read: SequenceManipulator {
    fn read(&mut self) -> Result<Self::Item, Self::In>;
}

trait Write: SequenceManipulator {
    fn write(&mut self, item: Self::Item) -> Result<(), Self::In>;
}

trait WriteRefInLong: SequenceManipulator {
    fn write_ref_in_long<'s, 'i: 's>(&'s mut self, item: &'i Self::Item) -> Result<(), Self::In>;
}

trait WriteRefIn: WriteRefInLong {
    fn write_ref_in(&mut self, item: &Self::Item) -> Result<(), Self::In>;
}

trait ReadRefInLong: SequenceManipulator {
    fn read_ref_in_long<'s, 'i: 's>(&'s mut self, item: &'i mut Self::Item) -> Result<(), Self::In>;
}

trait ReadRefIn: ReadRefInLong {
    fn read_ref_in(&mut self, item: &mut Self::Item) -> Result<(), Self::In>;
}

trait WriteRefOut: SequenceManipulator {
    fn write_ref_out(&mut self) -> Result<*mut Self::Item, Self::In>;
}

trait WriteRefOutLong: WriteRefOut {
    fn write_ref_out_long(&mut self) -> Result<&mut Self::Item, Self::In>;
}

trait ReadRefOut: SequenceManipulator {
    fn read_ref_out(&mut self) -> Result<*const Self::Item, Self::In>;
}

trait ReadRefOutLong: ReadRefOut {
    fn read_ref_out_long(&mut self) -> Result<&Self::Item, Self::In>;
}

trait WriteIn: SequenceManipulator {
    unsafe fn write_in(&mut self, item: *const Self::Item) -> Result<(), Self::In>;
}

trait ReadIn: SequenceManipulator {
    unsafe fn read_in(&mut self, item: *mut Self::Item) -> Result<(), Self::In>;
}

trait WriteOut: SequenceManipulator {
    unsafe fn write_out(&mut self) -> Result<*mut Self::Item, Self::In>;
}

trait ReadOut: SequenceManipulator {
    unsafe fn read_out(&mut self) -> Result<*const Self::Item, Self::In>;
}

trait FlushPrev: SequenceManipulator {
    fn flush_prev(&self) -> Result<(), Self::In>;
}

trait FlushNext: SequenceManipulator {
    fn flush_next(&self) -> Result<(), Self::In>;
}

trait SlurpPrev: SequenceManipulator {
    fn slurp_prev(&self) -> Result<(), Self::In>;
}

trait SlurpNext: SequenceManipulator {
    fn slurp_next(&self) -> Result<(), Self::In>;
}

trait WriteRefInLongMany1: WriteRefInLong {
    fn write_ref_in_long_many1<'s, 'i: 's>(&'s mut self, items: &'i Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait WriteRefInMany1: WriteRefInLongMany1 {
    fn write_ref_in_many1(&mut self, item: &Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait ReadRefInLongMany1: ReadRefInLong {
    fn read_ref_in_long_many1<'s, 'i: 's>(&'s mut self, item: &'i mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait ReadRefInMany1: ReadRefInLongMany1 {
    fn read_ref_in_many1(&mut self, item: &mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait WriteInMany1: WriteIn {
    unsafe fn write_in_many1(&mut self, item: *const Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait ReadInMany1: ReadIn {
    unsafe fn read_in_many1(&mut self, item: *mut Loaf<Self::Item>) -> Result<NonZeroUsize, Self::In>;
}

trait WriteRefOutMany1: WriteRefOut {
    fn write_ref_out_many1(&mut self) -> Result<*mut Loaf<Self::Item>, Self::In>;
}

trait WriteRefOutLongMany1: WriteRefOutMany1 {
    fn write_ref_out_long_many1(&mut self) -> Result<&mut Loaf<Self::Item>, Self::In>;
}

trait ReadRefOutMany1: ReadRefOut {
    fn read_ref_out_many1(&mut self) -> Result<*const Loaf<Self::Item>, Self::In>;
}

trait ReadRefOutLongMany1: ReadRefOutMany1 {
    fn read_ref_out_long_many1(&mut self) -> Result<&Loaf<Self::Item>, Self::In>;
}

trait WriteOutMany1: WriteOut {
    unsafe fn write_out_many1(&mut self) -> Result<*mut Loaf<Self::Item>, Self::In>;
}

trait ReadOutMany1: ReadOut {
    unsafe fn read_out_many1(&mut self) -> Result<*const Loaf<Self::Item>, Self::In>;
}

trait StopRead: SequenceManipulator {
    type StopR;
    fn stop_read(&self, reason: Self::StopR) -> Result<(), Self::In>;
}

trait StopWrite: SequenceManipulator {
    type StopW;
    fn stop_write(&self, reason: Self::StopW) -> Result<(), Self::In>;
}
