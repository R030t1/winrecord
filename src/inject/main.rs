#![windows_subsystem = "windows"]
#![feature(vec_into_raw_parts)]
extern crate winapi;
extern crate kernel32;
extern crate user32;

use std::ptr::*;
use std::mem::*;
use std::ffi::*;
use std::os::windows::ffi::OsStrExt;
use std::iter::*;

use kernel32::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;

fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(once(0))
        .collect()
}

fn rawinput_list_devices(
) {
}

unsafe extern "system" fn wndproc(
    hWnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> LRESULT {
    DefWindowProcW(hWnd, message, wParam, lParam)
}

pub fn main() {
    unsafe {
        let name = win32_string("inject");
        let title = win32_string("inject");

        let hinstance = GetModuleHandleW(null_mut()) as
            *mut HINSTANCE__;

        let wndclass = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: name.as_ptr()
        };

        RegisterClassW(&wndclass);
        let handle = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut()
        );

        let rc = ShowWindow(
            handle, SW_SHOW | SW_RESTORE
        );

        loop {
            let mut message: MSG = MaybeUninit::zeroed().assume_init();
            if GetMessageW(
                &mut message as *mut MSG,
                handle, 0, 0) > 0  {
                TranslateMessage(&message as *const MSG);
                DispatchMessageW(&message as *const MSG);
            } else {
                break;
            }
        }
    }
}