// DONET SOFTWARE
// Copyright (c) 2023, Donet Authors.

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3.
// You should have received a copy of this license along
// with this source code in a file named "LICENSE."
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program; if not, write to the Free Software Foundation,
// Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

#[path = "types.rs"]
mod type_aliases;

// Detect system endianness (byte order)
pub mod endianness {
    #[cfg(target_endian = "big")]
    pub fn swap_le_16(v: u16) -> u16 {
        return (v & 0x00ff) << 8 |
               (v & 0xff00) >> 8;
    }

    #[cfg(target_endian = "big")]
    pub fn swap_le_32(v: u32) -> u32 {
        return (v & 0x000000ff) << 24 |
               (v & 0x0000ff00) <<  8 |
               (v & 0x00ff0000) >>  8 |
               (v & 0xff000000) >> 24;
    }

    #[cfg(target_endian = "big")]
    pub fn swap_le_64(v: u64) -> u64 {
        return (v & 0x00000000000000ff) << 56 |
               (v & 0x000000000000ff00) << 40 |
               (v & 0x0000000000ff0000) << 24 |
               (v & 0x00000000ff000000) <<  8 |
               (v & 0x000000ff00000000) >>  8 |
               (v & 0x0000ff0000000000) >> 24 |
               (v & 0x00ff000000000000) >> 40 |
               (v & 0xff00000000000000) >> 56;
    }

    #[cfg(target_endian = "little")]
    pub fn swap_le_16(v: u16) -> u16 {
        return v; // no need to swap bytes
    }

    #[cfg(target_endian = "little")]
    pub fn swap_le_32(v: u32) -> u32 {
        return v;
    }

    #[cfg(target_endian = "little")]
    pub fn swap_le_64(v: u64) -> u64 {
        return v;
    }
}

#[allow(dead_code)] // FIXME: Remove once project matures
pub mod datagram {
    use crate::datagram::type_aliases::type_aliases as types;
    use crate::datagram::endianness;
    use std::vec::Vec;
    use std::result::Result; // not to be confused with std::io::Result

    type DgSize = u16;
    const DG_SIZE_MAX: DgSize = u16::MAX;

    // All possible errors that can be returned within Datagram's implementation.
    pub enum DgError {
        DatagramOverflow,
    }

    pub type DgResult = Result<(), DgError>;

    pub struct Datagram {
        buffer: Vec<u8>,
    }

    impl Datagram {
        pub fn new() -> Datagram {
            Datagram {
                buffer: Vec::new(),
            }
        }

        // Checks if we can add `length` number of bytes to the datagram.
        fn check_add_length(&self, length: DgSize) -> DgResult {
            let new_offset: usize = self.buffer.len() + usize::from(length);
            
            if new_offset > DG_SIZE_MAX.into() {
                // TODO: log error with more information
                return Err(DgError::DatagramOverflow);
            }
            return Ok(());
        }

        fn add_multiple_bytes(&mut self, bytes: DgSize, ) -> DgResult {
            let res: DgResult = self.check_add_length(bytes);
            if res.is_err() {
                return res;
            }

            return Ok(());
        }

        // Adds an unsigned 8-bit integer to the datagram that is
        // guaranteed to be one of the values 0x00 (false) or 0x01 (true).
        pub fn add_bool(&mut self, v: bool) -> DgResult {
            let mut res: DgResult = self.check_add_length(1);
            if res.is_err() {
                return res;
            }
            if v {
                res = self.add_u8(1);
            } else {
                res = self.add_u8(0);
            }
            return res;
        }
 
        // Adds an unsigned 8-bit integer value to the datagram.
        pub fn add_u8(&mut self, v: u8) -> DgResult {
            let res: DgResult = self.check_add_length(1);
            if res.is_err() {
                return res;
            }
            self.buffer.push(v);
            return Ok(());
        }

        pub fn add_u16(&mut self, mut v: u16) -> DgResult {
            let res: DgResult = self.check_add_length(2);
            if res.is_err() {
                return res;
            }
            v = endianness::swap_le_16(v);
            // FIXME: There is definitely a simpler way to do this.
            // Masking each byte and shifting it to the first byte,
            // then casting it as a u8 to represent one byte.
            self.buffer.push((v & 0xff00) as u8);
            self.buffer.push(((v & 0x00ff) << 8) as u8);
            return Ok(());
        }

        pub fn add_u32(&mut self, mut v: u32) -> DgResult {
            let res: DgResult = self.check_add_length(4);
            if res.is_err() {
                return res;
            }
            v = endianness::swap_le_32(v);
            self.buffer.push((v & 0xff000000) as u8);
            self.buffer.push(((v & 0x00ff0000) << 8) as u8);
            self.buffer.push(((v & 0x0000ff00) << 16) as u8);
            self.buffer.push(((v & 0x000000ff) << 24) as u8);
            return Ok(());
        }

        pub fn add_u64(&mut self, mut v: u64) -> DgResult {
            let res: DgResult = self.check_add_length(8);
            if res.is_err() {
                return res;
            }
            v = endianness::swap_le_64(v);
            self.buffer.push((v & 0xff00000000000000) as u8);
            self.buffer.push(((v & 0x00ff000000000000) << 8) as u8);
            self.buffer.push(((v & 0x0000ff0000000000) << 16) as u8);
            self.buffer.push(((v & 0x000000ff00000000) << 24) as u8);
            self.buffer.push(((v & 0x00000000ff000000) << 32) as u8);
            self.buffer.push(((v & 0x0000000000ff0000) << 40) as u8);
            self.buffer.push(((v & 0x000000000000ff00) << 48) as u8);
            self.buffer.push(((v & 0x00000000000000ff) << 56) as u8);
            return Ok(());
        }

        // signed integer aliases. same bitwise operations.
        pub fn add_i8(&mut self, v: i8) -> DgResult {
            return self.add_u8(v as u8);
        }

        pub fn add_i16(&mut self, v: i16) -> DgResult {
            return self.add_u16(v as u16);
        }

        pub fn add_i32(&mut self, v: i32) -> DgResult { 
            return self.add_u32(v as u32);
        }

        pub fn add_i64(&mut self, v: i64) -> DgResult { 
            return self.add_u64(v as u64);
        }
        
        // 32-bit IEEE 754 floating point. same bitwise operations.
        pub fn add_f32(&mut self, v: f32) -> DgResult {
            return self.add_u32(v as u32);
        }

        // 64-bit IEEE 754 floating point. same bitwise operations.
        pub fn add_f64(&mut self, v: f64) -> DgResult {
            return self.add_u64(v as u64);
        }

        // Adds a Datagram / Field length tag to the end of the datagram.
        pub fn add_size(&mut self, v: DgSize) -> DgResult {
            return self.add_u16(v as u16);
        }

        // Adds a 64-bit channel ID to the end of the datagram.
        pub fn add_channel(&mut self, v: types::Channel) -> DgResult {
            return self.add_u64(v as u64);
        }

        // Adds a 32-bit Distributed Object ID to the end of the datagram.
        pub fn add_doid(&mut self, v: types::DoId) -> DgResult {
            return self.add_u32(v as u32);
        }

        // Adds a 32-bit zone ID to the end of the datagram.
        pub fn add_zone(&mut self, v: types::Zone) -> DgResult {
            return self.add_u32(v as u32);
        }

        // Added for convenience, but also better performance
        // than adding the parent and the zone separately.
        pub fn add_location(&mut self, parent: types::DoId, zone: types::Zone) -> DgResult {
            let res: DgResult = self.add_u32(parent as u32);
            if res.is_err() {
                return res;
            }
            return self.add_u32(zone as u32);
        }

        // Adds raw bytes to the datagram via an unsigned 8-bit integer vector.
        // NOTE: not to be confused with add_blob(), which adds a dclass blob to the datagram.
        pub fn add_data(&mut self, mut v: Vec<u8>) -> DgResult {
            if v.len() > DG_SIZE_MAX.into() { // check input to avoid panic at .try_into() below
                return Err(DgError::DatagramOverflow); 
            }
            let res: DgResult = self.check_add_length(v.len().try_into().unwrap());
            if res.is_err() {
                return res;
            }
            self.buffer.append(&mut v);
            return Ok(());
        }

        // Appends another datagram's binary data to this datagram.
        pub fn add_datagram(&mut self, dg: Datagram) -> DgResult {
            let mut dg_buffer: Vec<u8> = dg.buffer;
            
            if dg_buffer.len() > DG_SIZE_MAX.into() {
                // Technically should not happen as the datagram given should
                // keep its buffer under the max dg size, but we should still handle
                // this error to avoid a panic at self.check_add_length().
                return Err(DgError::DatagramOverflow);
            }
            let res: DgResult = self.check_add_length(dg_buffer.len().try_into().unwrap());
            if res.is_err() {
                return res;
            }
            self.buffer.append(&mut dg_buffer);
            return Ok(());
        }

        // Adds a dclass string value to the end of the datagram.
        // A 16-bit length tag prefix with the string's size in bytes is added.
        pub fn add_string(&mut self, v: &str) -> DgResult {
            if v.len() > DG_SIZE_MAX.into() {
                // The string is too big to be described with a 16-bit length tag.
                return Err(DgError::DatagramOverflow);
            }
            let mut res: DgResult = self.add_u16(v.len().try_into().unwrap());
            if res.is_err() {
                return res; // couldn't fit length tag ;(
            }
            res = self.check_add_length(v.len().try_into().unwrap());
            if res.is_err() {
                return res; // can't fit the string ;(
            }
            // FIXME: i should've downloaded rust docs. sure there is a method
            // to convert a string to a byte array. this kid won't shut up
            // in my flight too so im reasonably pissed at the moment.
            // update: who tf turns their flash light on in the cabin at 1 AM???
            return Ok(());
        }

        // Adds a dclass blob value (binary data) to the end of the datagram.
        // A 16-bit length tag prefix with the blob's size in bytes is added.
        pub fn add_blob(&mut self, mut v: Vec<u8>) -> DgResult {
            let mut res: DgResult = self.add_size(v.len().try_into().unwrap());
            if res.is_err() {
                return res; // couldn't fit the length tag
            }
            res = self.check_add_length(v.len().try_into().unwrap());
            if res.is_err() {
                return res; // blob overflows datagram
            }
            self.buffer.append(&mut v);
            return Ok(());
        }
    }

    //pub struct DatagramIterator {

    //}
}
