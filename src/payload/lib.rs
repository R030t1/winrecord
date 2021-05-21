#[macro_use]
extern crate lazy_static;
extern crate winapi;
extern crate kernel32;
extern crate user32;

use std::ptr::*;
use std::mem::*;
use std::ffi::*;
use std::os::windows::ffi::OsStrExt;
use std::iter::*;

use std::sync::*;
use std::thread;

use lazy_static::*;

use widestring::*;

use kernel32::*;
use winapi::ctypes::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;
use winapi::um::minwinbase::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;
use winapi::shared::minwindef::{TRUE, DWORD};

#[no_mangle]
pub fn test() {
    println!("Hello, world!");
}

/*lazy_static! {
    static ref done: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
}*/

// TODO: Remove the fully qualified paths.
#[no_mangle]
extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: DWORD,
    reserved: std::ffi::c_void
) -> BOOL {
    match reason {
        // IDEA doesn't highlight properly, assumes variable.
        winnt::DLL_PROCESS_ATTACH => {
            println!("Attach.");
            thread::spawn(|| {
                println!("Thread.");
                let t = unsafe { GetCurrentProcessId() };
                std::fs::File::create(format!("C:\\Test\\{}", t))
            });
            TRUE
        },
        winnt::DLL_PROCESS_DETACH => {
            TRUE
        },
        _ => TRUE,
    }
}

// TODO: These can likely be all joined together depending on how we serialize info.
#[no_mangle]
extern "system" fn wndproc() { println!("wndproc") }
extern "system" fn cbtproc() { println!("cbtproc") }
extern "system" fn kbdproc() { println!("kbdproc") }
extern "system" fn mseproc() { println!("mseproc") }