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

#![feature(unicode)]

mod traits;

pub use traits::{Show, Format, SignPolicy, Utf8Write, FormattedInt, Rep, HEX};


/// Perform a cat which appends to an initial argument of type `String`.
#[macro_export] macro_rules! strcat {

    ($str:expr, $($args:tt)*) => {
        use ::std::mem;

        let mut v = mem::replace(&mut $str, String::new()).into_bytes();

        fcat!(v, $($args)*).ok();

        // Like with scat!, I'm not sure this should be the checked version. If we're appending to a
        // String this is going to make the complexity skyrocket.
        let s = match String::from_utf8(v) {
            Ok(s) => s,
            _     => panic!("strcat! macro generated invalid utf-8"),
        };

        mem::replace(&mut $str, s);
    }
}

/// Concatenate objects into strings.
/// 
/// # Examples
/// ```
/// # #[macro_use] extern crate cats;
/// # fn main() {
/// let s = scat!("Meow", ',', ' ', String::from("World"));
///
/// assert_eq!(s, "Meow, World");
/// # }
/// ```
#[macro_export] macro_rules! scat {
    ($($args:tt)*) => ({
        let len = cat_len!($($args)*);

        let mut buffer = Vec::with_capacity(len);

        cat_write!(&mut buffer, $($args)*).unwrap();

        // Here we're checking for valid utf-8, maybe this should be unchecked?
        match String::from_utf8(buffer) {
            Ok(s) => s,
            _     => panic!("scat! macro generated invalid utf-8"),
        }
    })
}

/// Return the length in bytes that a cat would create.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate cats;
///
/// # fn main() {
///  let len = cat_len!("Meow", ',', ' ', String::from("World"));
///
///  assert_eq!(len, "Meow, World".len());
/// # }
/// ```
#[macro_export] macro_rules! cat_len {
    ($($args:tt)*) => ({
        let mut total_len = 0;

        produce_len_code!(total_len, $($args)*)
    })
}

/// Write the 
#[macro_export] macro_rules! cat_write {
    ($buffer:expr, $($args:tt)*) => ({
        produce_write_code!(0, $buffer, $($args)*)
    })
}

#[macro_export] macro_rules! produce_len_code {
    ($len:expr, $fmt:expr ; $obj:expr) => ({
        $len += $crate::Format::len(&$fmt, &$obj);

        $len
    });

    ($len:expr, $fmt:expr ; $obj:expr, $($rest:tt)*) => ({
        $len += $crate::Format::len(&$fmt, &$obj);

        produce_len_code!($len, $($rest)*)
    });

    ($len:expr, $obj:expr) => ({
        $len += $crate::Show::len(&$obj);

        $len
    });

    ($len:expr, $obj:expr, $($rest:tt)*) => ({
        $len += $crate::Show::len(&$obj);

        produce_len_code!($len, $($rest)*)
    })
}

#[macro_export] macro_rules! produce_write_code {
    ($written:expr, $w:expr, $fmt:expr ; $obj:expr) => ({
        match $crate::Format::write(&$fmt, &$obj, $w) {
            Ok(n)  => Ok(n + $written),
            Err(e) => Err(e),
        }
    });

    ($written:expr, $w:expr, $fmt:expr ; $obj:expr, $($rest:tt)*) => ({
        match $crate::Format::write(&$fmt, &$obj, $w) {
            Ok(n)  => produce_write_code!(n + $written, $w, $($rest)*),
            Err(e) => Err(e),
        }
    });

    ($written:expr, $w:expr, $obj:expr) => ({
        match $crate::Show::write(&$obj, $w) {
            Ok(n)  => Ok(n + $written),
            Err(e) => Err(e),
        }
    });

    ($written:expr, $w:expr, $obj:expr, $($rest:tt)*) => ({
        match $crate::Show::write(&$obj, $w) {
            Ok(n)  => produce_write_code!(n + $written, $w, $($rest)*),
            Err(e) => Err(e),
        }
    })
}

#[macro_export] macro_rules! fcat {
    ($file:expr, $($args:tt)*) => ({
        use ::std::io::Write;
        $file.write_all(scat!($($args)*).as_bytes())
    })
}

#[macro_export] macro_rules! fcatln {
    ($file:expr, $($args:tt)*) => ({
        fcat!($file, $($args)*, '\n')
    })
}

#[macro_export] macro_rules! ecat {
    ($($args:tt)*) => ({
        fcat!(::std::io::stderr(), $($args)*).unwrap()
    })
}

#[macro_export] macro_rules! ecatln {
    ($($args:tt)*) => ({
        ecat!($($args)*, '\n')
    })
}

#[macro_export] macro_rules! cat {
    ($($args:tt)*) => ({
        fcat!(::std::io::stdout(),$($args)*).unwrap()
    })
}

#[macro_export] macro_rules! catln {
    ($($args:tt)*) => ({
        cat!($($args)*, '\n')
    })
}
