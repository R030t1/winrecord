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

unsafe fn rawinput_list_devices(
) {
    let mut sz: UINT = 0;
    let rc = GetRawInputDeviceList(
        null_mut(),
        &mut sz as *mut _,
        size_of::<RAWINPUTDEVICELIST>() as u32
    );

    let mut rids = vec![RAWINPUTDEVICELIST {
        hDevice: null_mut(),
        dwType: 0
    }; sz as usize].into_raw_parts();
    let rc = GetRawInputDeviceList(
        rids.0 as *mut _,
        &mut sz as *mut _,
        size_of::<RAWINPUTDEVICELIST>() as u32
    );

    let mut rids = Vec::from_raw_parts(
        rids.0, rids.1, rids.2
    );
    for i in 0..(sz as usize) {
        println!(
            "hDev: {} type: {}",
            rids[i].hDevice as usize,
            rids[i].dwType as usize
        );
    }
}

unsafe extern "system" fn wndproc(
    hWnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> LRESULT {
    match message {
        WM_CREATE => {
            DefWindowProcW(hWnd, message, wParam, lParam)
        },
        // TODO: Fix, below doesn't work.
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        },
        WM_QUIT => {
            std::process::exit(0)
        }
        _ => DefWindowProcW(hWnd, message, wParam, lParam)
    }
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

        let rc = AllocConsole();

        rawinput_list_devices();

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