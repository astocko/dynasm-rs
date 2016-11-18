pub mod x64;

use std::error::Error;
use std::io::{Read, Write};
use std::iter;
use std::process::{Command, Stdio};
use std::slice;


pub struct FormatStringIterator<'a> {
    inner: iter::Cloned<slice::Iter<'a, u8>>,
}

impl<'a> FormatStringIterator<'a> {
    pub fn new(buf: &'a [u8]) -> FormatStringIterator<'a> {
        FormatStringIterator { inner: buf.into_iter().cloned() }
    }
}

impl<'a> Iterator for FormatStringIterator<'a> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        if let Some(ty) = self.inner.next() {
            let size = self.inner.next().expect("Invalid format string data");
            Some((ty, size))
        } else {
            None
        }
    }
}

pub fn ndisasm(bytes: &[u8]) -> String {

    // echo -n -e '\xf3\x0f\x58\xc1' | ndisasm -b64 -

    let process = match Command::new("ndisasm")
        .args(&["-b", "64", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn() {
        Err(err) => panic!("couldn't spawn ndisasm: {}", err.description()),
        Ok(process) => process,
    };

    match process.stdin.unwrap().write_all(bytes) {
        Err(err) => panic!("Couldn't write to ndisasm stdin: {}", err.description()),
        Ok(_) => (),
    }

    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(err) => panic!("Couldn't read ndisasm stdout: {}", err.description()),
        Ok(_) => (),
    }

    let arr: Vec<&str> = s.split_at(28).1.split("\n").collect();
    return arr[0].trim().to_string();
}
