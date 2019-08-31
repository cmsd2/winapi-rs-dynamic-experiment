use std::mem;
use std::ptr;
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{
            INT,
            UINT,
            HINSTANCE,
            LPARAM,
            WPARAM,
            LRESULT,
            FALSE,
            TRUE,
        },
        ntdef:: {
            HANDLE,
        },
        windef::{
            HBRUSH,
            HWND,
            RECT,
        },
    },
    um::{
        winuser::{
            HWND_MESSAGE,
            WM_CLOSE,
            WM_DESTROY,
            WM_SETFONT,
            WNDCLASSEXW,
            CS_HREDRAW,
            CS_VREDRAW,
            SS_LEFT,
            SS_BITMAP,
            WS_CHILD,
            WS_VISIBLE,
            WS_EX_LEFT,
            IDC_ARROW,
            SPI_GETICONTITLELOGFONT,
            STM_SETIMAGE,
            IMAGE_BITMAP,
            COLOR_WINDOW,
            DestroyWindow,
            RegisterClassExW,
            LoadCursorW,
            DefWindowProcW,
            SystemParametersInfoForDpi,
            SendMessageW,
            SetWindowTextW,
            CreateWindowExW,
            LoadBitmapW,
            GetPropW,
            SetPropW,
        },
        winbase::{
            MulDiv,
        },
        wingdi::{
            LOGFONTW,
            CreateFontIndirectW,
        },
        commctrl::{
            SetWindowSubclass,
            DefSubclassProc,
        },
        libloaderapi::{
            GetModuleHandleW,
        },
    },
};
use hidpi::{
    get_dpi_for_system,
    get_window_font,
    make_l_param,
    get_parent_relative_window_rect,
    to_wstring,
    get_thread_dpi_awareness_context,
    awareness_context_to_str,
};

pub mod res;
use res::*;

pub const HWND_NAME_EXTERNAL: &'static str = "External Content";
pub const PLUGINWINDOWCLASSNAME: &'static str = "Plugin Window Class";
pub const DEFAULT_PADDING96: INT = 20;
pub const PROP_FONTSET: &'static str = "FONT_SET";
pub const DEFAULT_CHAR_BUFFER: usize = 200;

#[no_mangle]
pub extern "C" fn class_registration(h_inst: HINSTANCE)
{
    let plugin_window_class_name_str = to_wstring(PLUGINWINDOWCLASSNAME);
    let wcex = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: h_inst,
        hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
        hbrBackground: COLOR_WINDOW as HBRUSH,
        lpszClassName: plugin_window_class_name_str.as_ptr(),

        ..Default::default()
    };

    unsafe { RegisterClassExW(&wcex); }
}

pub fn scale_to_system_dpi(distance: INT, main_monitor_dpi: UINT) -> INT
{
    return unsafe { MulDiv(distance, main_monitor_dpi as INT, 96) } as INT
}

pub extern "system" fn wnd_proc(h_wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT
{
    match message {
        WM_CLOSE => {
            unsafe { DestroyWindow(h_wnd); }
            return 0;
        },

        WM_DESTROY => {
            return 0;
        },

        _ => {
            return unsafe { DefWindowProcW(h_wnd, message, w_param, l_param) }
        }
    }
}

// This method will create an HWND tree that is scaled to the system DPI
// ("System DPI" is a global DPI that is based off of the scale factor of the primary display).
// When the process that this code is running in is has a DPI_AWARENESS_CONTEXT of 
// DPI_AWARENESS_CONTEXT_UNAWARE, the system DPI will be 96
#[no_mangle]
pub extern "C" fn create_content_hwnd(h_instance: HINSTANCE, n_width: INT, n_height: INT) -> HWND
{
    // Register the window class
    class_registration(h_instance);

    // Get the "System DPI"
    // Don't do this in per-monitor aware code as this will either
    // return 96 or the system DPI but will not return the per-monitor DPI
    let main_monitor_dpi = get_dpi_for_system();

    // Create an HWND tree that is parented to the message window (HWND_MESSAGE)
    let plugin_window_class_name_str = to_wstring(PLUGINWINDOWCLASSNAME);
    let hwnd_name_external_str = to_wstring(HWND_NAME_EXTERNAL);
    let h_wnd_external_content = unsafe { CreateWindowExW(0, 
        plugin_window_class_name_str.as_ptr(), 
        hwnd_name_external_str.as_ptr(), 
        WS_VISIBLE | WS_CHILD, 0, 0, n_width, n_height, 
        HWND_MESSAGE, ptr::null_mut(), 
        h_instance, ptr::null_mut()) };
            
    // Add some child controls
    let static_str = to_wstring("STATIC");
    let static_name_str = to_wstring("External content static (text) control");
    let h_wnd_static = unsafe { CreateWindowExW(WS_EX_LEFT, static_str.as_ptr(), static_name_str.as_ptr(), SS_LEFT | WS_CHILD | WS_VISIBLE,
        scale_to_system_dpi(DEFAULT_PADDING96, main_monitor_dpi), 
        scale_to_system_dpi(DEFAULT_PADDING96, main_monitor_dpi), 
        scale_to_system_dpi(n_width - 2*DEFAULT_PADDING96, main_monitor_dpi),
        scale_to_system_dpi(75, main_monitor_dpi),
        h_wnd_external_content, ptr::null_mut(), h_instance, ptr::null_mut()) };
    
    // Subclass the static control so that we can ignore WM_SETFONT from the host
    unsafe { SetWindowSubclass(h_wnd_static, Some(SubclassProc), 0, 0); }

    // Set the font for the static control
    let _h_font_old = get_window_font(h_wnd_static);
    let mut lf_text = LOGFONTW::default();
    unsafe { SystemParametersInfoForDpi(SPI_GETICONTITLELOGFONT, mem::size_of::<LOGFONTW>() as u32, &mut lf_text as *mut LOGFONTW as *mut c_void, FALSE as u32, main_monitor_dpi); }
    let h_font_new = unsafe { CreateFontIndirectW(&lf_text) };
    if h_font_new != ptr::null_mut()
    {
        unsafe {
            SendMessageW(h_wnd_static, WM_SETFONT, h_font_new as usize, make_l_param(TRUE as u16, 0) as isize);
        }
    }

    // Convert DPI awareness context to a string

    let dpi_awareness_context = get_thread_dpi_awareness_context();

    let awareness_context = awareness_context_to_str(dpi_awareness_context);
    
    // Build the output string
    let wnd_text = to_wstring(format!("HWND content from an external source. The thread that created this content had a thread context of {}, with a DPI of: {}", awareness_context, main_monitor_dpi));
    unsafe {
        SetWindowTextW(h_wnd_static, wnd_text.as_ptr());
    }

    // Load a bitmap
    let module_name_str = to_wstring("dll_plugin.dll");
    let h_mod = unsafe { GetModuleHandleW(module_name_str.as_ptr()) };
    let h_bmp = unsafe { LoadBitmapW(h_mod, IDB_BITMAP1 as *const u16) };
    if h_bmp == ptr::null_mut()
    {
        // Out of memory
        return ptr::null_mut();
    }

    // Create a static control to put the image in to
    let mut rc_client = RECT::default();
    get_parent_relative_window_rect(h_wnd_static, &mut rc_client);
    let static_str = to_wstring("STATIC");
    let static_name_str = to_wstring("External content static (bitmap) control");
    let h_wnd_image = unsafe { CreateWindowExW(WS_EX_LEFT, static_str.as_ptr(), static_name_str.as_ptr(), SS_BITMAP | WS_CHILD | WS_VISIBLE,
        scale_to_system_dpi(DEFAULT_PADDING96, main_monitor_dpi),
        rc_client.bottom + scale_to_system_dpi(DEFAULT_PADDING96, main_monitor_dpi),
        scale_to_system_dpi(n_width - 2 * DEFAULT_PADDING96, main_monitor_dpi),
        scale_to_system_dpi(200, main_monitor_dpi),
        h_wnd_external_content, ptr::null_mut(), h_instance, ptr::null_mut()) };
    unsafe {
        SendMessageW(h_wnd_image, STM_SETIMAGE, IMAGE_BITMAP as usize, h_bmp as LPARAM);
    }

    return h_wnd_external_content;
}

// Subclass the static control so that the parent can't send a new font when
// the DPI changes. We want to illustrate how a child HWND can be bitmap
// stretched by Windows. If the font were reset it would detract from 
// illustrating this.
#[no_mangle]
pub extern "system" fn SubclassProc(h_wnd: HWND, u_msg: UINT,
    w_param: WPARAM, l_param: LPARAM, _u_id_sub_class: usize,
    _dw_ref_data: usize) -> LRESULT
{
    match u_msg {
        // Store a flag indicating that the font has been set, then
        // don't let the font be set after that
        WM_SETFONT => {
            let prop_str = to_wstring(PROP_FONTSET);
            let b_font_set = unsafe { GetPropW(h_wnd, prop_str.as_ptr()) };
            if b_font_set == ptr::null_mut()
            {
                // Allow the font set to happen
                unsafe { SetPropW(h_wnd, prop_str.as_ptr(), TRUE as HANDLE); }
                return unsafe { DefSubclassProc(h_wnd, u_msg, w_param, l_param) };
            }
            else
            {
                return 0;
            }
        },

        _ => {
            return unsafe { DefSubclassProc(h_wnd, u_msg, w_param, l_param) };
        }
    }
}
