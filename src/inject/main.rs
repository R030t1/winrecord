#![windows_subsystem = "windows"]
#![feature(vec_into_raw_parts)]
extern crate winapi;
extern crate kernel32;
extern crate user32;

use clap::{Arg, App};

use std::ptr::*;
use std::mem::*;
use std::ffi::*;
use std::os::windows::ffi::OsStrExt;
use std::iter::*;

use std::io;

use widestring::*;

use kernel32::*;
use winapi::ctypes::*;
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
    unsafe {
        let mut sz: UINT = 0;
        let _rc = GetRawInputDeviceList(
            null_mut(),
            &mut sz as *mut _,
            size_of::<RAWINPUTDEVICELIST>() as u32
        );

        let rids = vec![RAWINPUTDEVICELIST {
            hDevice: null_mut(),
            dwType: u32::MAX
        }; sz as usize].into_raw_parts();
        let _rc = GetRawInputDeviceList(
            rids.0 as *mut _,
            &mut sz as *mut _,
            size_of::<RAWINPUTDEVICELIST>() as u32
        );

        let rids = Vec::from_raw_parts(
            rids.0, rids.1, rids.2
        );
        for i in 0..(sz as usize) {
            println!(
                "hDev: {:?} type: {:?}",
                rids[i].hDevice,
                rids[i].dwType
            );

            let mut ssz: UINT = 0;
            let _rc = GetRawInputDeviceInfoW(
                rids[i].hDevice,
                RIDI_DEVICENAME,
                null_mut(),
                &mut ssz as *mut _
            );

            let name = vec![0 as wchar_t; ssz as usize]
                .into_raw_parts();
            let _rc = GetRawInputDeviceInfoW(
                rids[i].hDevice,
                RIDI_DEVICENAME,
                name.0 as *mut _,
                &mut ssz as *mut _
            );

            let name = Vec::from_raw_parts(name.0, name.1, name.2);
            let n = U16CString::from_vec_with_nul(name)
                .unwrap();
            println!("{}", n.to_string_lossy());

            let mut di: RID_DEVICE_INFO = MaybeUninit::zeroed().assume_init();
            let mut ssz = size_of::<RID_DEVICE_INFO>() as UINT;
            let _rc = GetRawInputDeviceInfoW(
                rids[i].hDevice,
                RIDI_DEVICEINFO,
                &mut di as *mut _ as *mut _,
                &mut ssz as *mut _
            );

            if di.dwType == RIM_TYPEHID {
                let mut ssz: UINT = 0;
                let _rc = GetRawInputDeviceInfoW(
                    rids[i].hDevice,
                    RIDI_PREPARSEDDATA,
                    null_mut(),
                    &mut ssz as *mut _
                );
                println!("RIDI_PREPARSEDDATA ssz: {}", ssz);

                let ppd = vec![0 as u8; ssz as usize]
                    .into_raw_parts();
                let _rc = GetRawInputDeviceInfoW(
                    rids[i].hDevice,
                    RIDI_PREPARSEDDATA,
                    ppd.0 as *mut _,
                    &mut ssz as *mut _
                );

                let ppd = Vec::from_raw_parts(ppd.0, ppd.1, ppd.2);
                println!("{:?}", ppd);
            }

            match di.dwType {
                RIM_TYPEMOUSE => {
                    println!(
                        "{} {} {} {}",
                        di.u.mouse().dwId,
                        di.u.mouse().dwNumberOfButtons,
                        di.u.mouse().dwSampleRate,
                        di.u.mouse().fHasHorizontalWheel
                    )
                },
                RIM_TYPEKEYBOARD => {
                    println!(
                        "{} {} {} {} {} {}",
                        di.u.keyboard().dwType,
                        di.u.keyboard().dwSubType,
                        di.u.keyboard().dwKeyboardMode,
                        di.u.keyboard().dwNumberOfFunctionKeys,
                        di.u.keyboard().dwNumberOfIndicators,
                        di.u.keyboard().dwNumberOfKeysTotal
                    )
                },
                RIM_TYPEHID => {
                    println!(
                        "{} {} {} {} {}",
                        di.u.hid().dwVendorId,
                        di.u.hid().dwProductId,
                        di.u.hid().dwVersionNumber,
                        di.u.hid().usUsagePage,
                        di.u.hid().usUsage
                    )
                },
                _ => ()
            }
        }
    }
}

unsafe fn windowshook_setup(title: &str) {
    // TODO: Occasionally won't find module (or procedure) with generic message.
    let payload = LoadLibraryW(
        U16String::from_os_str("payload.dll").as_ptr()
    );
    println!("{:?}", payload);
    if payload == MaybeUninit::zeroed().assume_init() {
        println!("{:?}", io::Error::last_os_error())
    }

    let wndproc = GetProcAddress(
        payload, OsStr::new("wndproc").to_str().unwrap().as_ptr() as *const i8
    );
    println!("{:?}", wndproc);
    if wndproc == MaybeUninit::zeroed().assume_init() {
        println!("{:?}", io::Error::last_os_error());
    }

    println!("Looking for \"{}\".", title);
    let h = FindWindowW(
        null_mut(),
        U16CString::from_os_str("").unwrap().as_ptr()
    );
    println!("Return: {:?}", h);

    let t = GetWindowThreadProcessId(h, null_mut());
    let hwp = SetWindowsHookExW(
        WH_CALLWNDPROC,
        transmute::<*const std::ffi::c_void, HOOKPROC>(wndproc),
        payload as *mut _,
        t
    );
    println!("SetWindowsHookEx: {:?}", hwp);

    /*transmute::<
        *const std::ffi::c_void,
        unsafe extern "system" fn() -> ()
    >(wndproc)();*/


}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    match message {
        WM_CREATE => {
            DefWindowProcW(hwnd, message, wparam, lparam)
        },
        // TODO: Fix, below doesn't work.
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        },
        WM_QUIT => {
            std::process::exit(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

pub fn main() {
    let matches = App::new("inject")
        .version("0.1.0")
        .author("Sid Spry")
        .about("Inject windows desktop hook DLL into program")
        .arg(Arg::with_name("title")
            .help("Title of window to hook")
            .short("t")
            .long("title")
            .value_name("TITLE")
            .takes_value(true)
            .required(true))
        .get_matches();

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

        let _rc = RegisterClassW(&wndclass);
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

        let _rc = ShowWindow(
            handle, SW_SHOW | SW_RESTORE
        );
        let _rc = AllocConsole();

        rawinput_list_devices();

        windowshook_setup(matches.value_of("title").unwrap());

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