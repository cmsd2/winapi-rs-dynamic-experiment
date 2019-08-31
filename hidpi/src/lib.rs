pub mod win;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::ptr;
use std::io;
use libc::size_t;
use winapi::{
    ctypes::wchar_t,
	shared::{
        minwindef::{
            BOOL,
            TRUE,
            FALSE,
            UINT,
            INT,
            WPARAM,
            LPARAM,
            LRESULT,
            HINSTANCE,
         },
        windef::{
            POINT,
            RECT,
            HWND,
            HFONT,
            HBRUSH,
            DPI_AWARENESS,
            DPI_AWARENESS_INVALID,
            DPI_AWARENESS_SYSTEM_AWARE,
            DPI_AWARENESS_PER_MONITOR_AWARE,
            DPI_AWARENESS_UNAWARE,
            DPI_AWARENESS_CONTEXT,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            DPI_AWARENESS_CONTEXT_UNAWARE,
            DPI_HOSTING_BEHAVIOR,
        }
    },
	um::{
        winuser::{
            self,
            HWND_DESKTOP,
        },
        wingdi,
    },
};

pub const GA_PARENT: UINT = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DpiAwareness {
	Unaware,
	System,
	PerMonitor,
	Other,
}

pub fn make_l_param(l: u16, h: u16) -> u32 {
    (l as u32) | ((h as u32) << 16)
}

pub fn is_process_dpi_aware() -> Option<bool> {
	win::is_process_dpi_aware()
}

pub fn get_process_dpi_awareness() -> io::Result<Option<DpiAwareness>> {
	Ok(win::get_process_dpi_awareness()?.map(|awareness| {
		match awareness {
			win::WinDpiAwareness::ProcessDpiUnaware => DpiAwareness::Unaware,
			win::WinDpiAwareness::ProcessSystemDpiAware => DpiAwareness::System,
			win::WinDpiAwareness::ProcessPerMonitorDpiAware => DpiAwareness::PerMonitor,
			win::WinDpiAwareness::Unknown(_o) => DpiAwareness::Other,
		}
	}))
}

pub fn set_process_dpi_aware() -> Option<bool> {
	win::set_process_dpi_aware()
}

pub fn set_process_dpi_awareness(awareness: DpiAwareness) -> io::Result<bool> {
	let win_awareness = match awareness {
		DpiAwareness::Unaware => Ok(win::WinDpiAwareness::ProcessDpiUnaware),
		DpiAwareness::System => Ok(win::WinDpiAwareness::ProcessSystemDpiAware),
		DpiAwareness::PerMonitor => Ok(win::WinDpiAwareness::ProcessPerMonitorDpiAware),
		_ => Err(io::Error::new(io::ErrorKind::Other, format!("unsupported dpi awareness type {:?}", awareness)))
	}?;

	win::set_process_dpi_awareness(win_awareness)
}

#[no_mangle]
pub extern "C" fn get_parent_relative_window_rect(h_wnd: HWND, child_bounds: *mut RECT) -> BOOL
{
    if FALSE == unsafe { winuser::GetWindowRect(h_wnd, child_bounds) }
    {
        return FALSE;
    }
    
    unsafe { winuser::MapWindowPoints(HWND_DESKTOP, winuser::GetAncestor(h_wnd, GA_PARENT), child_bounds as *mut POINT, 2); }

    return TRUE;
}

#[no_mangle]
pub extern "C" fn get_stock_brush(brush: UINT) -> HBRUSH {
    unsafe { wingdi::GetStockObject(brush as INT) as HBRUSH }
}

#[no_mangle]
pub extern "C" fn get_hinstance_for_h_wnd(h_wnd: HWND) -> HINSTANCE {
    unsafe { winuser::GetWindowLongW(h_wnd, winuser::GWL_HINSTANCE) as HINSTANCE }
}

#[no_mangle]
pub extern "C" fn get_window_font(h_wnd: HWND) -> HFONT {
    unsafe  {  winuser::SendMessageW(h_wnd, winuser::WM_GETFONT, 0, 0) as HFONT }
}

#[no_mangle]
pub extern "C" fn set_window_font(h_wnd: HWND, h_font: HFONT, f_redraw: BOOL) -> LRESULT {
    unsafe { winuser::SendMessageW(h_wnd, winuser::WM_SETFONT, h_font as WPARAM, f_redraw as LPARAM) }
}

#[no_mangle]
pub extern "C" fn get_dpi_for_system() -> UINT {
    unsafe { winuser::GetDpiForSystem() }
}

#[no_mangle]
pub extern "C" fn get_dpi_for_window(h_wnd: HWND) -> UINT {
    unsafe { winuser::GetDpiForWindow(h_wnd) }
}

pub fn get_maybe_dpi_by_awareness(h_wnd: HWND) -> Option<UINT> {
    match get_thread_dpi_awareness() {
        DPI_AWARENESS_SYSTEM_AWARE => {
            Some(get_dpi_for_system())
        },
        DPI_AWARENESS_PER_MONITOR_AWARE => {
            Some(get_dpi_for_window(h_wnd))
        },
        _ => None
    }
}

#[no_mangle]
pub extern "C" fn get_dpi_by_awareness(h_wnd: HWND, ret: *mut UINT) -> BOOL {
    if let Some(dpi) = get_maybe_dpi_by_awareness(h_wnd) {
        unsafe {
            *ret = dpi;
            TRUE
        }
    } else {
        FALSE
    }
}

#[no_mangle]
pub extern "C" fn are_dpi_awareness_contexts_equal(a: DPI_AWARENESS_CONTEXT, b: DPI_AWARENESS_CONTEXT) -> BOOL {
    win::are_dpi_awareness_contexts_equal(a, b)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no supported"))
        .expect("are_dpi_awareness_contexts_equal")
}

#[no_mangle]
pub extern "C" fn get_thread_dpi_awareness_context() -> DPI_AWARENESS_CONTEXT {
    win::get_thread_dpi_awareness_context()
        .unwrap_or(DPI_AWARENESS_CONTEXT_UNAWARE)
}

#[no_mangle]
pub extern "C" fn get_thread_dpi_awareness() -> DPI_AWARENESS {
    win::get_thread_dpi_awareness_context()
        .and_then(win::get_awareness_from_dpi_awareness_context)
        .unwrap_or(DPI_AWARENESS_UNAWARE)
}

#[no_mangle]
pub extern "C" fn get_awareness_from_dpi_awareness_context(context: DPI_AWARENESS_CONTEXT) -> DPI_AWARENESS {
    win::get_awareness_from_dpi_awareness_context(context)
        .unwrap_or(DPI_AWARENESS_UNAWARE)
}

#[no_mangle]
pub extern "C" fn get_thread_dpi_hosting_behavior() -> DPI_HOSTING_BEHAVIOR {
    win::get_thread_dpi_hosting_behavior().unwrap()
}

#[no_mangle]
pub extern "C" fn set_thread_dpi_hosting_behavior(behavior: DPI_HOSTING_BEHAVIOR) -> DPI_HOSTING_BEHAVIOR {
    win::set_thread_dpi_hosting_behavior(behavior).unwrap()
}

#[no_mangle]
pub extern "C" fn set_thread_dpi_awareness_context(context: DPI_AWARENESS_CONTEXT) -> DPI_AWARENESS_CONTEXT {
    win::set_thread_dpi_awareness_context(context).unwrap()
}

pub fn awareness_to_str(awareness: DPI_AWARENESS) -> &'static str {
    match awareness {
        DPI_AWARENESS_INVALID => {
           "DPI_AWARENESS_INVALID"
        },
        DPI_AWARENESS_SYSTEM_AWARE => {
            "DPI_AWARENESS_SYSTEM_AWARE"
        },
        DPI_AWARENESS_PER_MONITOR_AWARE => {
            "DPI_AWARENESS_PER_MONITOR_AWARE"
        },
        _ => {
            "DPI_AWARENESS_UNAWARE"
        }
    }
}

#[no_mangle]
pub extern "C" fn format_awareness(lpwstr: *mut wchar_t, len: size_t, awareness: DPI_AWARENESS) -> i32 {
    string_copy(lpwstr, len, awareness_to_str(awareness));
    0
}

pub fn awareness_context_to_str(context: DPI_AWARENESS_CONTEXT) -> &'static str {
    let awareness = get_awareness_from_dpi_awareness_context(context);
    match awareness {
        DPI_AWARENESS_SYSTEM_AWARE => {
            "DPI_AWARENESS_CONTEXT_SYSTEM_AWARE"
        },
        DPI_AWARENESS_PER_MONITOR_AWARE => {
            if let Some(isequal) = win::are_dpi_awareness_contexts_equal(context, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) {
                if isequal == TRUE {
                    "DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2"
                } else {
                    "DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE"
                }
            } else { 
                "DPI_AWARENESS_CONTEXT_UNAWARE"
            }
        },
        _ => {
            "DPI_AWARENESS_CONTEXT_UNAWARE"
        }
    }
}

#[no_mangle]
pub extern "C" fn format_awareness_context(lpwstr: *mut wchar_t, len: size_t, context: DPI_AWARENESS_CONTEXT) -> i32 {
    string_copy(lpwstr, len, awareness_context_to_str(context));
    0
}

pub fn to_wstring<S>(s: S) -> Vec<u16> where S: AsRef<str> {
    OsStr::new(s.as_ref())
        .encode_wide()
        .chain(once(0))
        .collect()
}

pub extern "C" fn string_copy(lpwstr: *mut wchar_t, capacity: size_t, s: &str) -> size_t {
    let w: Vec<u16> = to_wstring(s);
    let len = std::cmp::min(capacity - 1, w.len());
    
    unsafe {
        ptr::copy_nonoverlapping(w.as_ptr(), lpwstr, len + 1);
    }

    len
}