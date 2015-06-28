// Copyright (c) 2015, Sam Payson
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
// associated documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute,
// sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
// NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::cmp;
use std::io;
use std::num::Wrapping;

/// A trait for types that know how to display themselves.
pub trait Show {
    /// How many bytes will the utf8-encoded string representation of `self` take?
    fn len(&self) -> usize;

    /// Write the string resentation of `self` to `w`. The number of bytes written must be exactly
    /// the same as the number returned by `self.len()`.
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize>;
}

/// A trait for types that know how to format another type.
pub trait Format<T> {
    /// How many bytes will the utf8-encoded string representation of `t` formatted by `self` take?
    fn len(&self, t: &T) -> usize;

    /// Write the string resentation of `t` formatted by `self` to `w`. The number of bytes
    /// written must be exactly the same as the number returned by `self.len(t)`.
    fn write<W: io::Write>(&self, t: &T, w: &mut W) -> io::Result<usize>;
}

pub struct Utf8Write<'x, W: io::Write + 'x>(pub &'x mut W);

impl<'x, W: io::Write + 'x> Utf8Write<'x, W> {
    fn push(&mut self, c: char) -> io::Result<usize> {
        let mut buf = [0u8; 4];
        let limit = c.encode_utf8(&mut buf).unwrap();

        self.0.write_all(&buf[0..limit]).map(|()| limit)
    }

    fn push_str(&mut self, s: &str) -> io::Result<usize> {
        self.0.write_all(s.as_bytes()).map(|()| s.len())
    }
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

    fn write<W: io::Write>(&self, x: &u64, w: &mut W) -> io::Result<usize> {
        let mut written = 0;

        let base = self.digits.len() as u64;

        let mut utf8_w = Utf8Write(w);

        // Pad with the zero digit until the minimum width is reached.
        let padding = self.min_len - cmp::min(self.num_digits(*x), self.min_len);

        written += match self.sign {
            SignPolicy::Plus  => try!(utf8_w.push('+')),
            SignPolicy::Space => try!(utf8_w.push(' ')),
            SignPolicy::Empty => 0,
        };

        written += try!(utf8_w.push_str(self.prefix));

        for _ in 0..padding {
            written += try!(utf8_w.push(self.digits[0]));
        }

        let mut r = self.reverse(*x);

        if r == 0 {
            written += try!(utf8_w.push(self.digits[0]));
        } else {
            for _ in 0..self.num_digits(*x) {
                written += try!(utf8_w.push(self.digits[(r % base) as usize]));
                r /= base;
            }
        }

        Ok(written + try!(utf8_w.push_str(self.suffix)))
    }
}

impl<'x> Format<u32> for FormattedInt<'x> {
    fn len(&self, x: &u32) -> usize { Format::len(self, &(*x as u64)) }
    fn write<W: io::Write>(&self, x: &u32, w: &mut W) -> io::Result<usize> {
        Format::write(self, &(*x as u64), w)
    }
}

impl<'x> Format<u16> for FormattedInt<'x> {
    fn len(&self, x: &u16) -> usize { Format::len(self, &(*x as u64)) }
    fn write<W: io::Write>(&self, x: &u16, w: &mut W) -> io::Result<usize> {
        Format::write(self, &(*x as u64), w)
    }
}

impl<'x> Format<u8> for FormattedInt<'x> {
    fn len(&self, x: &u8) -> usize { Format::len(self, &(*x as u64)) }
    fn write<W: io::Write>(&self, x: &u8, w: &mut W) -> io::Result<usize> {
        Format::write(self, &(*x as u64), w)
    }
}

impl<'x> Format<usize> for FormattedInt<'x> {
    fn len(&self, x: &usize) -> usize { Format::len(self, &(*x as u64)) }
    fn write<W: io::Write>(&self, x: &usize, w: &mut W) -> io::Result<usize> {
        Format::write(self, &(*x as u64), w)
    }
}

impl<'x> Format<i64> for FormattedInt<'x> {
    fn len(&self, x: &i64) -> usize {
        match self.sign {
            SignPolicy::Empty if *x < 0 => Format::len(self, &(x.abs() as u64)) + 1,
            _                           => Format::len(self, &(x.abs() as u64)),
        }
    }

    fn write<W: io::Write>(&self, x: &i64, w: &mut W) -> io::Result<usize> {
        if *x < 0 {
            Ok(try!(Utf8Write(w).push('-')) +
                try!(Format::write(&FormattedInt {
                    sign: SignPolicy::Empty,
                    .. *self
                }, &(x.abs() as u64), w)))
        } else {
            Format::write(self, &(*x as u64), w)
        }
    }
}

const DECIMAL_DIGITS: &'static [char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const HEX_DIGITS: &'static [char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                                       'a', 'b', 'c', 'd', 'e', 'f'];

pub const HEX: FormattedInt<'static> = FormattedInt {
    prefix:  "",
    suffix:  "",
    digits:  HEX_DIGITS,
    min_len: 0,
    sign:    SignPolicy::Empty,
};

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

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Format::write(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self, w)
    }
}

impl Show for u32 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as u64), w)
    }
}

impl Show for u16 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as u64), w)
    }
}

impl Show for u8 {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as u64), w)
    }
}

impl Show for usize {
    fn len(&self) -> usize { Show::len(&(*self as u64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as u64), w)
    }
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

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Format::write(&FormattedInt {
            prefix:  "",
            suffix:  "",
            digits:  DECIMAL_DIGITS,
            min_len: 0,
            sign:    SignPolicy::Empty,
        }, self, w)
    }
}

impl Show for i32 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as i64), w)
    }
}

impl Show for i16 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as i64), w)
    }
}

impl Show for i8 {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as i64), w)
    }
}

impl Show for isize {
    fn len(&self) -> usize { Show::len(&(*self as i64)) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(&(*self as i64), w)
    }
}

impl Show for str {
    fn len(&self) -> usize { self.len() }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Utf8Write(w).push_str(self)
    }
}

impl<'x, T: ?Sized> Show for &'x T where T: Show {
    fn len(&self) -> usize { Show::len(*self) }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Show::write(*self, w)
    }
}

impl Show for String {
    fn len(&self) -> usize { self.len() }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Utf8Write(w).push_str(&self[..])
    }
}

impl Show for char {
    fn len(&self) -> usize { self.len_utf8() }
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        Utf8Write(w).push(*self)
    }
}

impl<'x, T: ?Sized, U> Format<U> for &'x T where T: Format<U> {
    fn len(&self, u: &U) -> usize { Format::len(*self, u) }
    fn write<W: io::Write>(&self, u: &U, w: &mut W) -> io::Result<usize> {
        Format::write(*self, u, w)
    }
}

pub struct Rep(pub usize);

impl<T> Format<T> for Rep
where T: Show {
    fn len(&self, t: &T) -> usize { self.0 * Show::len(t) }
    fn write<W: io::Write>(&self, t: &T, w: &mut W) -> io::Result<usize> {
        let mut len = 0;
        for _ in 0..self.0 {
            len += try!(Show::write(t, w));
        }

        Ok(len)
    }
}

impl<T> Show for Option<T>
where T: Show {
    fn len(&self) -> usize {
        match self {
            &Some(ref t) => t.len(),
            &None        => 0,
        }
    }

    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        match self {
            &Some(ref t) => t.write(w),
            &None        => Ok(0),
        }
    }
}
