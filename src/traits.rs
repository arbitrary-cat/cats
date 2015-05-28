// Copyright (c) 2015, Sam Payson
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::cmp;
use std::num::Wrapping;

/// A trait for types that know how to display themselves.
pub trait Show {
    /// How many bytes will the utf8-encoded string representation of `self` take?
    fn len(&self) -> usize;

    /// Write the string resentation of `self` to `buf`. The number of bytes written must be exactly
    /// the same as the number returned by `self.len()`.
    fn write(&self, buf: &mut String);
}

/// A trait for types that know how to format another type.
pub trait Format<T> {
    /// How many bytes will the utf8-encoded string representation of `t` formatted by `self` take?
    fn len(&self, t: &T) -> usize;

    /// Write the string resentation of `t` formatted by `self` to `buf`. The number of bytes
    /// written must be exactly the same as the number returned by `self.len(t)`.
    fn write(&self, t: &T, buf: &mut String);
}

/// What should be printed before a positive integer?
pub enum SignPolicy {
    /// Print a '+' sign before positive numbers, as in "+372"
    Plus,

    /// Print a ' ' before positive numbers, as in " 372"
    Space,

    /// Don't print anything before positive numbers, as in "372"
    Empty,
}

pub struct FormattedInt<'x> {
    pub prefix:  &'x str,
    pub suffix:  &'x str,
    pub digits:  &'x [char],
    pub min_len: usize,
    pub sign:    SignPolicy,
}

impl<'x> FormattedInt<'x> {
    fn sign_len(&self) -> usize {
        match self.sign {
            SignPolicy::Plus | SignPolicy::Space => 1,
            SignPolicy::Empty                    => 0,
        }
    }

    fn with_fanciness(&self, s: usize) -> usize {
        let padded = if s < self.min_len { self.min_len } else { s };

        padded + self.prefix.len() + self.suffix.len() + self.sign_len()
    }

    fn num_digits(&self, x: u64) -> usize {
        let mut length = 1;
        let base       = Wrapping(self.digits.len() as u64);
        let mut limit  = base;

        while base * limit > limit {
            if limit > Wrapping(x) { return length; }

            length = length + 1;
            limit  = limit  * base;
        }

        length
    }

    fn reverse(&self, mut x: u64) -> u64 {
        let mut r = 0;
        let base  = self.digits.len() as u64;

        while x != 0 {
            r = (r * base) + (x % base);
            x = x / base;
        }

        r
    }
}

impl<'x> Format<u64> for FormattedInt<'x> {
    // TODO: This assumes that all digits require 1 byte to encode.
    fn len(&self, x: &u64) -> usize {
        self.with_fanciness(self.num_digits(*x))
    }

    fn write(&self, x: &u64, buf: &mut String) {
        let base = self.digits.len() as u64;

        // Pad with the zero digit until the minimum width is reached.
        let padding = self.min_len - cmp::min(self.num_digits(*x), self.min_len);

        match self.sign {
            SignPolicy::Plus  => buf.push('+'),
            SignPolicy::Space => buf.push(' '),
            SignPolicy::Empty => (),
        }

        buf.push_str(self.prefix);

        for _ in 0..padding {
            buf.push(self.digits[0]);
        }

        let mut r = self.reverse(*x);

        if r == 0 {
            buf.push(self.digits[0]);
        } else {
            while r != 0 {
                buf.push(self.digits[(r % base) as usize]);
                r /= base;
            }
        }

        buf.push_str(self.suffix);
    }
}

impl<'x> Format<u32> for FormattedInt<'x> {
    fn len(&self, x: &u32) -> usize { Format::len(self, &(*x as u64)) }
    fn write(&self, x: &u32, buf: &mut String) { Format::write(self, &(*x as u64), buf) }
}

impl<'x> Format<u16> for FormattedInt<'x> {
    fn len(&self, x: &u16) -> usize { Format::len(self, &(*x as u64)) }
    fn write(&self, x: &u16, buf: &mut String) { Format::write(self, &(*x as u64), buf) }
}

impl<'x> Format<u8> for FormattedInt<'x> {
    fn len(&self, x: &u8) -> usize { Format::len(self, &(*x as u64)) }
    fn write(&self, x: &u8, buf: &mut String) { Format::write(self, &(*x as u64), buf) }
}

impl<'x> Format<usize> for FormattedInt<'x> {
    fn len(&self, x: &usize) -> usize { Format::len(self, &(*x as u64)) }
    fn write(&self, x: &usize, buf: &mut String) { Format::write(self, &(*x as u64), buf) }
}

impl<'x> Format<i64> for FormattedInt<'x> {
    fn len(&self, x: &i64) -> usize {
        match self.sign {
            SignPolicy::Empty if *x < 0 => Format::len(self, &(x.abs() as u64)) + 1,
            _                           => Format::len(self, &(x.abs() as u64)),
        }
    }

    fn write(&self, x: &i64, buf: &mut String) {
        if *x < 0 {
            buf.push('-');
            Format::write(&FormattedInt {
                sign: SignPolicy::Empty,
                .. *self
            }, &(x.abs() as u64), buf);
        } else {
            Format::write(self, &(*x as u64), buf);
        }
    }
}

static DECIMAL_DIGITS: &'static [char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

impl Show for u64 {
    fn len(&self) -> usize {
        Format::len(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self)
    }

    fn write(&self, buf: &mut String) {
        Format::write(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self, buf)
    }
}

impl Show for u32 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as u64), buf) }
}

impl Show for u16 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as u64), buf) }
}

impl Show for u8 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as u64), buf) }
}

impl Show for usize {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as u64), buf) }
}

impl Show for i64 {
    fn len(&self) -> usize {
        Format::len(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self)
    }

    fn write(&self, buf: &mut String) {
        Format::write(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self, buf)
    }
}

impl Show for i32 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as i64), buf) }
}

impl Show for i16 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as i64), buf) }
}

impl Show for i8 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as i64), buf) }
}

impl Show for isize {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write(&self, buf: &mut String) { Show::write(&(*self as i64), buf) }
}

impl Show for str {
    fn len(&self) -> usize { self.len() }
    fn write(&self, buf: &mut String) { buf.push_str(self) }
}

impl<'x, T: ?Sized> Show for &'x T where T: Show {
    fn len(&self) -> usize { Show::len(*self) }
    fn write(&self, buf: &mut String) { Show::write(*self, buf) }
}

impl Show for String {
    fn len(&self) -> usize { self.len() }
    fn write(&self, buf: &mut String) { buf.push_str(&self[..]) }
}

impl Show for char {
    fn len(&self) -> usize { self.len_utf8() }
    fn write(&self, buf: &mut String) { buf.push(*self) }
}
