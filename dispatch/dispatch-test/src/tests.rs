#![allow(dead_code)]
//! Test module.

use ::std::{
    fs::File,
    io::{Stderr, Stdout, Write},
};

use ::katalog_lib_dispatch::Dispatch;

#[derive(Debug)]
struct First {}
impl First {
    pub fn into_run(self) -> u8 {
        1
    }
}

#[derive(Debug)]
struct Second(u8);
impl Second {
    pub fn into_run(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Dispatch)]
#[dispatch(impl {
    dispatch fn <Self>::into_run(self) -> u8 {
        0
    }
})]
#[dispatch(impl {
    #[dispatch(map(Some))]
    dispatch as opt_run fn <Self>::into_run(self) -> Option<u8> {
        None
    }
})]
enum Cases {
    First(First),
    Second {
        item: Second,
    },
    Third {
        #[dispatch(ignore)]
        third: u8,
    },
    Fourth,
}

#[derive(Dispatch)]
#[dispatch(impl {
    dispatch for<T> as run1 fn run(self) {
        ()
    }
})]
enum T1 {}

#[derive(Dispatch)]
#[dispatch {
    as write_wrap fn Write::write(&mut self, buf: &[u8]) -> ::std::io::Result<usize>;
}]
enum Writer {
    File(File),
    Stdout(Stdout),
    Stderr(Stderr),
    Vec(Vec<u8>),
}
