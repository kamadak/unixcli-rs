//
// Copyright (c) 2016 KAMADA Ken'ichi.
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

//! Provides access to the name of the current program.

use std::env;
use std::ffi::{OsStr, OsString};
use std::mem;
use std::path::Path;
use std::sync::{Arc, Mutex, Once, ONCE_INIT};

struct Outer {
    inner: Mutex<Inner>,
}

struct Inner {
    name: Arc<Option<OsString>>,
}

static mut PROGNAME: *const Outer = 0 as *const Outer;
static ONCE: Once = ONCE_INIT;

fn init_mutex() {
    ONCE.call_once(|| {
        let instance = Outer {
            inner: Mutex::new(Inner { name: Arc::new(get_exec_name()) }),
        };
        unsafe {
            PROGNAME = mem::transmute(Box::new(instance));
        }
    });
}

/// Returns the name of the current program as a `String`.
/// An empty string is returned if any error occurred.
pub fn getprogname() -> String {
    // Explicit dereference to call Option::as_ref instead of Arc::as_ref.
    (*getprogname_arc()).as_ref()
        .and_then(|s| s.to_str()).unwrap_or("").to_string()
    // (*getprogname_arc()).as_ref()
    //     .map(|s| s.to_string_lossy().into_owned()).unwrap_or("".to_string())
}

/// Returns the name of the current program.
/// In contrast to `getprogname()`, this function returns a
/// shared `OsString`.
pub fn getprogname_arc() -> Arc<Option<OsString>> {
    init_mutex();
    let inner = unsafe { &(*PROGNAME).inner };
    inner.lock().unwrap().name.clone()
}

/// Sets the name of the current program.
/// The name is taken from the last component of the path.
pub fn setprogname<S>(path: &S) where S: AsRef<OsStr> + ?Sized {
    init_mutex();
    let name = Path::new(path).file_name().unwrap_or(path.as_ref()).to_owned();
    let new = Arc::new(Some(name));
    let inner = unsafe { &(*PROGNAME).inner };
    inner.lock().unwrap().name = new;
}

fn get_exec_name() -> Option<OsString> {
    env::current_exe().ok().and_then(
        |pb| pb.file_name().map(|s| s.to_owned()))
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    // These tests cannot run concurrently because they change the
    // global state.
    #[test]
    fn all() {
        initial();
        basic();
        old_in_use();
        components();
        threads();
    }

    fn initial() {
        assert!(getprogname().starts_with("unixcli"));
    }

    fn basic() {
        setprogname("test");
        assert_eq!(getprogname(), "test");
        assert_eq!(getprogname(), "test");
        setprogname("test2");
        assert_eq!(getprogname(), "test2");
        assert_eq!(getprogname(), "test2");
    }

    fn old_in_use() {
        setprogname("test3");
        let arc = getprogname_arc();
        setprogname("test4");
        assert_eq!(getprogname(), "test4");
        assert_eq!(*arc, Some(OsString::from("test3")));
    }

    fn components() {
        setprogname("/path/to/a/command");
        assert_eq!(getprogname(), "command");
    }

    fn threads() {
        let child1 = thread::spawn(move || {
            for i in 0..10000 {
                setprogname(&format!("child1: {}", i));
            }
        });
        let child2 = thread::spawn(move || {
            for i in 0..10000 {
                setprogname(&format!("child2: {}", i));
            }
        });
        for _ in 0..10000 {
            println!("{}", getprogname());
        }
        child1.join().unwrap();
        child2.join().unwrap();
    }
}
