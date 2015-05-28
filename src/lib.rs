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

mod traits;

pub use traits::*;

/// Concatenate objects into strings.
/// 
/// # Examples
/// ```
/// #[macro_use] extern crate cat;
///
/// fn main() {
///     let s = cat!('(', 'a', ')', String::from(" "), 12, " + ", 7, " = ", 12 + 7);
///
///     assert_eq!(s, "(a) 12 + 7 = 19");
/// }
/// ```
#[macro_export] macro_rules! cat {
    ($($([$fmt:expr])* $obj:expr),*) => ({
        let len = cat_len!($($([$fmt])* $obj),*);

        let mut buffer = String::with_capacity(len);

        cat_write!(&mut buffer, $($([$fmt])* $obj),*);

        buffer
    })
}

#[macro_export] macro_rules! cat_len {
    ($($([$fmt:expr])* $obj:expr),*) => ({
        let mut total_len = 0;
        $(
            produce_len_code!(total_len, $($fmt,)* $obj);
        )*

        total_len
    })
}


#[macro_export] macro_rules! cat_write {
    ($buffer:expr, $($([$fmt:expr])* $obj:expr),*) => {
        $(
            produce_write_code!($buffer, $($fmt,)* $obj);
        )*
    }
}

#[macro_export] macro_rules! produce_len_code {
    ($len:expr, $fmt:expr, $obj:expr) => {
        $len += $crate::Format::len(&$fmt, &$obj);
    };
    ($len:expr, $obj:expr) => {
        $len += $crate::Show::len(&$obj);
    }
}

#[macro_export] macro_rules! produce_write_code {
    ($str:expr, $fmt:expr, $obj:expr) => {
        $crate::Format::write(&$fmt, &$obj, $str);
    };

    ($str:expr, $obj:expr) => {
        $crate::Show::write(&$obj, $str);
    }
}

#[macro_export] macro_rules! fcat {
    ($file:expr, $($([$fmt:expr])* $obj:expr),*) => ({
        $file.write_all(cat!($($([$fmt])* $obj),*).as_bytes())
    })
}
