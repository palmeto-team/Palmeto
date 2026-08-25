// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module provides a static,
//          fixed-capacity ring buffer.
//
//          A ring buffer uses a single,
//          fixed-size array connected end-to-end,
//          allowing data streaming (without memory management)
//

use crate::core::status::{KResult, Status};
pub struct RingBuffer<const CAPACITY: usize>
{
    data: [u8; CAPACITY],
    head: usize,
    tail: usize,
    full: bool,
}

impl<const CAPACITY: usize> Default for RingBuffer<CAPACITY>
{
    ///
    /// This routine returns a default, empty RingBuffer
    /// It will invoke 'Self::new()'
    ///
    fn default() -> Self
    {
        Self::new()
    }
}

impl<const CAPACITY: usize> RingBuffer<CAPACITY>
{
    ///
    /// This routine consturcts a new RingBuffer structure,
    /// it will  be completely empty,
    /// besides setting the maximum capacity to the given one.
    ///
    pub const fn new() -> Self
    {
        const
        {
            assert!(CAPACITY > 0,
                    "RINGBUFFER: CAPACITY NOT GREATER THAN ZERO");
        }
        //
        // Ring buffers start cleared
        // head, tail = 0
        // full       = false
        //
        Self
        {
            data: [0; CAPACITY],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    ///
    /// This routine writes a slice of bytes to the buffer,
    /// this will overwrite the oldest data in the buffer,
    /// if the buffer is still full of course.
    ///
    /// # Arguments
    ///
    /// * bytes - A byte slice of data to write
    ///
    pub fn write(&mut self, bytes: &[u8]) -> KResult<()>
    {
        if bytes.is_empty() 
        {
            return Ok(());
        }

        let mut overflowed = false;

        let count = bytes.len();

        if count > CAPACITY 
        {
            let skip = count - CAPACITY;
            let bytes_to_copy = &bytes[skip..];
            
            self.data.copy_from_slice(bytes_to_copy);
            self.head = 0;
            self.tail = 0;
            self.full = true;
            
            return Err(Status::BUFFER_OVERFLOW);
        }

        let available_space = CAPACITY - self.len();

        if count > available_space 
        {
            overflowed = true;
        }

        if self.head >= self.tail && !self.full 
        {
            let first_chunk_len = CAPACITY - self.head;

            if count <= first_chunk_len 
            {
                self.data[self.head..self.head + count].copy_from_slice(&bytes[..count]);
                self.head = (self.head + count) % CAPACITY;
            } else {
                let second_chunk_len = count - first_chunk_len;

                self.data[self.head..CAPACITY].copy_from_slice(&bytes[..first_chunk_len]);
                self.data[..second_chunk_len].copy_from_slice(&bytes[first_chunk_len..count]);
                self.head = second_chunk_len;
            }

        } else {
            self.data[self.head..self.head + count].copy_from_slice(&bytes[..count]);
            self.head += count;
        }

        if overflowed 
        {
            self.tail = self.head;
            self.full = true;

            Err(Status::BUFFER_OVERFLOW)
        } else {
            if self.head == self.tail 
            {
                self.full = true;
            }
            
            Ok(())
        }
    }

    ///
    /// This routines pushes a single byte into the buffer,
    /// if the buffer is already full it will return from the function
    ///
    /// # Arguments
    ///
    /// * byte - The single byte to push into the buffer
    ///
    pub fn push(&mut self, byte: u8) -> KResult<()>
    {

        if self.full
        {
            //
            // The buffer is full,
            // We cannot add anything more..
            //
            Err(Status::BUFFER_OVERFLOW)
        }
        else {
            
            self.data[self.head] = byte;
            self.head            = (self.head + 1 ) % CAPACITY;

            if self.head == self.tail
            {
                //
                // HEAD == TAIL
                // This means the buffer is full
                //
                self.full = true;
            }

            Ok(())
        }
    }

    ///
    /// This routine reads the available bytes from the buffer,
    /// and place them into the destination,
    /// it will return the number of bytes that were read.
    ///
    /// # Arguments
    ///
    /// * dest - Byte slice where read data will be stored
    ///
    pub fn read(&mut self, dest: &mut [u8]) -> usize
    {
        let available = self.len();

        if available == 0 || dest.is_empty()
        {
            return 0;
        }

        let count = core::cmp::min(available, dest.len());

        if self.tail < self.head
        {
            dest[..count].copy_from_slice(&self.data[self.tail..self.tail + count]);
        }else {
            let first_chunk_len = CAPACITY - self.tail;

            if count <= first_chunk_len
            {
                dest[..count].copy_from_slice(&self.data[self.tail..self.tail + count]);
            } else {
                let second_chunk_len = count - first_chunk_len;

                dest[..first_chunk_len].copy_from_slice(&self.data[self.tail..CAPACITY]);
                dest[first_chunk_len..count].copy_from_slice(&self.data[..second_chunk_len]);            
            }
        }

        self.tail = (self.tail + count) % CAPACITY;
        self.full = false;

        count
    }

    ///
    /// This routine will completely clear the buffer.
    /// Use if you no longer need the stored data
    /// but you want to reuse the buffer.
    ///
    pub fn clear(&mut self)
    {
        //
        // Completely clear the ring buffer
        // 'head, tail' = 0
        // 'full'       = false
        //
        self.head = 0;
        self.tail = 0;
        self.full = false;
    }

    ///
    /// This routine returns the number of bytes in the buffer.
    ///
    pub fn len(&self) -> usize
    {
        if self.full
        {
            CAPACITY
        } else if self.head >= self.tail
        {
            self.head - self.tail
        } else {
            CAPACITY - self.tail + self.head
        }
    }

    ///
    /// This routine will return if the buffer is empty or not.
    ///
    pub fn is_empty(&self) -> bool
    {
        !self.full && self.head == self.tail
    }

    ///
    /// This routine will return if the buffer is full or not.
    ///
    pub fn full(&self) -> bool
    {
        self.full
    }
    
    ///
    /// This routine will return the maximum capacity of the buffer.
    ///
    pub fn capacity(&self) -> usize
    {
        CAPACITY
    }
}

impl<const CAPACITY: usize> core::fmt::Write for RingBuffer<CAPACITY> {
    ///
    /// This routine will write a string slice into the buffer
    /// converting it into bytes so that it works.
    ///
    /// # Arguments
    ///
    /// * s - the string to write
    ///
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self.write(s.as_bytes())
        {
            Ok(_) | Err(Status::BUFFER_OVERFLOW) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}