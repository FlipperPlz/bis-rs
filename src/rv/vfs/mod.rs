use std::sync::{Once, Mutex};

static INIT: Once = Once::new();
static mut FILESYSTEM: *const Mutex<RvFilesystem> = 0 as *const Mutex<RvFilesystem>;

pub fn get_filesystem() -> &'static Mutex<RvFilesystem> {
    unsafe {
        INIT.call_once(|| {
            FILESYSTEM = Box::into_raw(Box::new(Mutex::new(RvFilesystem)));
        });
        &*FILESYSTEM
    }
}

struct RvFilesystem;