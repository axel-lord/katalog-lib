//! Test module.

use ::katalog_lib_dispatch::Dispatch;

#[derive(Dispatch)]
#[dispatch(impl {
    for<T> as run1 fn run(self) {
        ()
    }
})]
enum T1 {}
