use std::marker::PhantomData;
use super::act::Act;
use super::Batch;
use super::iterator::BatchIterator;
use super::packet_batch::cast_from_u8;
use super::super::interface::EndOffset;
use super::super::pmd::*;
use super::super::interface::Result;

pub struct ParsedBatch<'a, T: 'a + EndOffset, V>
    where V: 'a + Batch + BatchIterator + Act
{
    parent: &'a mut V,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, V> Act for ParsedBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self
    }

    fn done(&mut self) -> &mut Self {
        self.parent.done();
        self
    }

    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }
}

batch!{ParsedBatch, [parent: &'a mut V], [phantom: PhantomData]}

impl<'a, T, V> BatchIterator for ParsedBatch<'a, T, V>
    where T: 'a + EndOffset,
          V: 'a + Batch + BatchIterator + Act
{
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        let address = self.parent.payload(idx);
        let offset = T::offset(cast_from_u8::<T>(address));
        address.offset(offset as isize)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        let parent_payload = self.parent.next_payload(idx);
        match parent_payload {
            Some((packet, idx)) => {
                let offset = T::offset(cast_from_u8::<T>(packet));
                Some((packet.offset(offset as isize), idx))
            }
            None => None,
        }
    }
}