#![allow(dead_code)]
//! Test module.

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
    fn <Self>::into_run(self) -> u8 {
        0
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
    for<T> as run1 fn run(self) {
        ()
    }
})]
enum T1 {}
