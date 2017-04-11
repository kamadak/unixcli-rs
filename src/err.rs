//
// Copyright (c) 2017 KAMADA Ken'ichi.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.
//

//! Prints messages to the standard error output.

use std::fmt;
use std::io;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::str;

use progname;

/// Prints the formatted message to the standard error output (stderr)
/// and terminates the program with the given `status` value.
/// The program name, a colon, and a space are output before the message,
/// which is followed by a newline character.
#[macro_export]
macro_rules! err {
    ($status:expr, $($fmtargs:tt)*) => ({
        warn!($($fmtargs)*);
        ::std::process::exit($status);
    })
}

/// Prints the formatted message to the standard error output (stderr).
/// The program name, a colon, and a space are output before the message,
/// which is followed by a newline character.
#[macro_export]
macro_rules! warn {
    ($fmt:expr) => (
        $crate::err::vwarn(format_args!(concat!($fmt, "\n")));
    );
    ($fmt:expr, $($args:tt)*) => (
        $crate::err::vwarn(format_args!(concat!($fmt, "\n"), $($args)*));
    );
}

/// This function is not a part of public/stable APIs.
/// This function should be used through `err!` or `warn!` macros.
pub fn vwarn(fmt: fmt::Arguments) {
    let mut buf = Vec::new();
    if let Some(ref os) = *progname::getprogname_arc() {
        #[cfg(unix)]
        buf.extend_from_slice(os.as_bytes());
        #[cfg(not(unix))]
        match os.to_str() {
            Some(s) => { let _ = write!(&mut buf, "{}", s); },
            None => {},
        };
    }
    buf.extend_from_slice(b": ");
    let msgstart = buf.len();
    let _ = buf.write_fmt(fmt);
    if let Err(e) = io::stderr().write(&buf) {
        // The message was composed by write_fmt, so from_utf8 should not fail.
        let msg = str::from_utf8(&buf[msgstart..]).unwrap_or("");
        // If writing to stderr failed, writing the panic message will
        // also fail, but anyway...
        panic!("failed to write to stderr: {}: {}", e, msg);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn warn() {
        warn!("warn 1");
        warn!("warn {}", 2);
    }
}
