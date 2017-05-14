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
    ($status:expr, $fmt:expr) => (
        $crate::err::verr($status, format_args!(concat!($fmt, "\n")));
    );
    ($status:expr, $fmt:expr, $($args:tt)*) => (
        $crate::err::verr(
            $status, format_args!(concat!($fmt, "\n"), $($args)*));
    );
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
/// This function should be used through the `err!` macro.
pub fn verr(status: i32, fmt: fmt::Arguments) {
    vwarn(fmt);
    tester::exit(status);
}

/// This function is not a part of public/stable APIs.
/// This function should be used through the `warn!` macro.
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
    if let Err(e) = tester::stderr().write(&buf) {
        // The message was composed by write_fmt, so from_utf8 should not fail.
        let msg = str::from_utf8(&buf[msgstart..]).unwrap_or("");
        // If writing to stderr failed, writing the panic message will
        // also fail, but anyway...
        panic!("failed to write to stderr: {}: {}", e, msg);
    }
}

#[cfg(not(test))]
mod tester {
    #[inline(always)]
    pub fn exit(status: i32) { ::std::process::exit(status); }
    #[inline(always)]
    pub fn stderr() -> ::std::io::Stderr { ::std::io::stderr() }
}

#[cfg(test)]
mod tester {
    pub fn exit(status: i32) { panic!("expected exit with {}", status); }
    pub fn stderr() -> DummyStderr { DummyStderr::new() }

    use std::cell::RefCell;
    use std::io;
    use std::io::Result;
    thread_local!(static STDERR_BUF: RefCell<Vec<u8>> = {
        RefCell::new(Vec::new())
    });
    pub struct DummyStderr();
    impl DummyStderr {
        pub fn new() -> DummyStderr { DummyStderr() }
    }
    impl io::Write for DummyStderr {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            STDERR_BUF.with(|v| v.borrow_mut().extend_from_slice(buf));
            Ok(buf.len())
        }
        fn flush(&mut self) -> Result<()> { Ok(()) }
    }
    pub fn get_stderr() -> Vec<u8> {
        STDERR_BUF.with(|v| v.borrow().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The status 0 is a bit dangerous.  The test runner assumes the
    // tests are successful if the exit status is 0, even when the
    // dummy exit is not working and the process really exits before
    // completing the tests,
    #[test]
    #[should_panic(expected = "expected exit with 0")]
    fn err1() {
        err!(0, "err 1");
    }

    #[test]
    #[should_panic(expected = "expected exit with 9")]
    fn err2() {
        err!(9, "err {}", 2);
    }

    #[test]
    fn warn() {
        warn!("warn 1");
        assert!(tester::get_stderr().ends_with(b": warn 1\n"));
        warn!("warn {}", 2);
        assert!(tester::get_stderr().ends_with(b": warn 2\n"));
        warn!("{} {}", "warn", 3);
        assert!(tester::get_stderr().ends_with(b": warn 3\n"));
    }
}
