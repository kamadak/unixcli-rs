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

#[macro_use(err, errp)]
extern crate unixcli;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use unixcli::progname;

fn main() {
    let mut args = env::args_os();
    progname::setprogname(args.next().unwrap().as_os_str());

    let args = match args.len() {
        0 => args.chain(vec![OsStr::new("-").to_os_string()]),
        _ => args.chain(vec![]),
    };
    for path in args {
        let path = Path::new(path.as_os_str());
        if let Err(e) = cat_one(path) {
            if path == Path::new("-") {
                err!(1, "stdin: {}", e);
            } else {
                errp!(1, path, "{}", e);
            }
        }
    }
}

fn cat_one(path: &Path) -> io::Result<()> {
    if path == Path::new("-") {
        cat_read(io::stdin())
    } else {
        cat_read(try!(File::open(path)))
    }
}

fn cat_read<R>(r: R) -> io::Result<()> where R: Read {
    let mut reader = BufReader::new(r);
    let mut buf = vec![0; 64 * 1024];
    loop {
        match try!(reader.read(&mut buf)) {
            0 => return Ok(()),
            n => { try!(io::stdout().write(&buf[0..n])); },
        }
    }
}
