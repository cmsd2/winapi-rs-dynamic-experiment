#![windows_subsystem = "windows"]

extern crate hidpi;

use std::ptr;
use std::mem;
use std::cell::RefCell;
use libc::{
    size_t,
};
use winapi::{
    ctypes::{
        c_void,
        wchar_t,
    },
	shared::{
        minwindef::{
            BOOL,
            TRUE,
            FALSE,
            UINT,
            INT,
            WPARAM,
            LPARAM,
            HIWORD,
            LOWORD,
            LRESULT,
            MAX_PATH,
            HINSTANCE,
         },
        ntdef::HANDLE,
        windef::{
            RECT,
            HWND,
            HMENU,
            HBRUSH,
            DPI_AWARENESS_CONTEXT,
            DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            DPI_AWARENESS_CONTEXT_UNAWARE,
            DPI_HOSTING_BEHAVIOR,
            DPI_HOSTING_BEHAVIOR_MIXED,
        }
    },
	um::{
        winuser,
        winbase,
        wingdi,
        commdlg,
        winnt::{
            LPWSTR,
            LPCWSTR,
        },
        consoleapi,
        wincon,
        libloaderapi,
    },
};

use hidpi::{  
    get_window_font,
    set_window_font,
    make_l_param,
    get_parent_relative_window_rect,
    to_wstring,
    get_stock_brush,
    get_thread_dpi_awareness,
    get_thread_dpi_awareness_context,
    set_thread_dpi_awareness_context,
    set_thread_dpi_hosting_behavior,
    get_maybe_dpi_by_awareness,
    get_hinstance_for_h_wnd,
    awareness_to_str,
    awareness_context_to_str,
};

pub const WINDOW_WIDTH96: INT = 500;
pub const WINDOW_HEIGHT96: INT = 700;
pub const DEFAULT_CHAR_BUFFER: size_t = 150;
pub const DEFAULT_PADDING96: UINT = 20;
pub const DEFAULT_BUTTON_HEIGHT96: INT = 25;
pub const DEFAULT_BUTTON_WIDTH96: INT = 100;
pub const SAMPLE_STATIC_HEIGHT96: UINT = 50;
pub const WINDOWCLASSNAME: &'static str = "SetThreadDpiAwarenessContextSample";
pub const HWND_NAME_CHECKBOX: &'static str = "CHECKBOX";
pub const HWND_NAME_RADIO: &'static str = "RADIO";
pub const HWND_NAME_STATIC: &'static str = "Static";
pub const HWND_NAME_DIALOG: &'static str = "Open a System Dialog";
pub const HWND_NAME_EXTERNAL: &'static str = "External Content";
pub const PLUGINWINDOWCLASSNAME: &'static str = "Plugin Window Class";
pub const EXTERNAL_CONTENT_WIDTH96: INT = 400;
pub const EXTERNAL_CONTENT_HEIGHT96: INT = 400;
pub const GA_PARENT: UINT = 1;
pub const PROP_DPIISOLATION: &'static str = "PROP_ISOLATION";

pub mod res;
use res::*;

#[repr(C)]
pub struct CreateParams
{
    pub b_enable_non_client_dpi_scaling: BOOL,
    pub b_child_window_dpi_isolation: BOOL,
}

impl Default for CreateParams { 
    fn default() -> Self {
        CreateParams {
            b_enable_non_client_dpi_scaling: FALSE,
            b_child_window_dpi_isolation: FALSE,
        }
    }
}

thread_local! {
    pub static CREATE_PARAMS: RefCell<CreateParams> = RefCell::new(CreateParams::default());
}

#[no_mangle]
pub extern "C" fn update_dpi_string(h_wnd: HWND, u_dpi: UINT) {
    let awareness = get_thread_dpi_awareness();
    let context = get_thread_dpi_awareness_context();

    let s = format!("DPI Awareness: {}\nDPI Awareness Context: {}\nGetDpiForWindow(.....): {}",
        awareness_to_str(awareness),
        awareness_context_to_str(context),
        u_dpi
    );

    unsafe {
        winuser::SetWindowTextW(h_wnd, to_wstring(&s).as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn update_and_dpi_scale_child_windows(h_wnd: HWND, u_dpi: UINT) {
    let h_wnd_dialog: HWND;
    let h_wnd_radio: HWND;

    let u_padding = unsafe { winbase::MulDiv(DEFAULT_PADDING96 as i32, u_dpi as i32, 96) };
    let mut rc_client = RECT { left: 0, bottom: 0, top: 0, right: 0 };
    unsafe { winuser::GetClientRect(h_wnd, &mut rc_client); }

    let static_str = to_wstring("STATIC");
    let h_wnd_static = unsafe { winuser::FindWindowExW(h_wnd, ptr::null_mut(), static_str.as_ptr(), ptr::null()) };
    if h_wnd_static == ptr::null_mut()
    {
        return;
    }

    let u_width = (rc_client.right - rc_client.left) - 2 * u_padding;
    let u_height = unsafe { winbase::MulDiv(SAMPLE_STATIC_HEIGHT96 as i32, u_dpi as i32, 96) };
    unsafe { winuser::SetWindowPos(
        h_wnd_static,
        ptr::null_mut(),
        u_padding,
        u_padding,
        u_width,
        u_height,
        winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE); }

    update_dpi_string(h_wnd_static, u_dpi);

    // Size and position the checkbox
    let button_str = to_wstring("BUTTON");
    let check_box_str = to_wstring(HWND_NAME_CHECKBOX);
    let h_wnd_checkbox = unsafe { winuser::FindWindowExW(h_wnd, ptr::null_mut(), button_str.as_ptr(), check_box_str.as_ptr()) };
    if h_wnd_checkbox == ptr::null_mut()
    {
        return;
    }
    get_parent_relative_window_rect(h_wnd_static, &mut rc_client);
    unsafe { winuser::SetWindowPos(
        h_wnd_checkbox,
        ptr::null_mut(),
        u_padding,
        rc_client.bottom + u_padding,
        winbase::MulDiv(DEFAULT_BUTTON_WIDTH96, u_dpi as i32, 96),
        winbase::MulDiv(DEFAULT_BUTTON_HEIGHT96, u_dpi as i32, 96), winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE); }

    // Size and position the radio button
    let radio_str = to_wstring(HWND_NAME_RADIO);
    h_wnd_radio = unsafe {  winuser::FindWindowExW(h_wnd, ptr::null_mut(), button_str.as_ptr(), radio_str.as_ptr()) };
    if h_wnd_checkbox == ptr::null_mut()
    {
        return;
    }
    get_parent_relative_window_rect(h_wnd_checkbox, &mut rc_client);
    unsafe { winuser::SetWindowPos(h_wnd_radio, ptr::null_mut(), rc_client.right + u_padding, rc_client.top,
        winbase::MulDiv(DEFAULT_BUTTON_WIDTH96, u_dpi as i32, 96),
        winbase::MulDiv(DEFAULT_BUTTON_HEIGHT96, u_dpi as i32, 96),
        winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE); }

    // Size and position the dialog button
    let dialog_str = to_wstring(HWND_NAME_DIALOG);
    h_wnd_dialog = unsafe { winuser::FindWindowExW(h_wnd, ptr::null_mut(), button_str.as_ptr(), dialog_str.as_ptr()) };
    get_parent_relative_window_rect(h_wnd_checkbox, &mut rc_client);
    unsafe { winuser::SetWindowPos(h_wnd_dialog, ptr::null_mut(), u_padding, rc_client.bottom + u_padding,
        winbase::MulDiv(DEFAULT_BUTTON_WIDTH96 * 2, u_dpi as i32, 96), // Make this one twice as wide as the others
        winbase::MulDiv(DEFAULT_BUTTON_HEIGHT96, u_dpi as i32, 96),
        winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE); }

    // Size and position the external content HWND
    let window_class_name_str = to_wstring(PLUGINWINDOWCLASSNAME);
    let h_wnd_name_external = to_wstring(HWND_NAME_EXTERNAL);
    let  h_wnd_external = unsafe { winuser::FindWindowExW(h_wnd, ptr::null_mut(), window_class_name_str.as_ptr(), h_wnd_name_external.as_ptr()) };
    get_parent_relative_window_rect(h_wnd_dialog, &mut rc_client);
    unsafe { winuser::SetWindowPos(h_wnd_external, h_wnd_dialog, u_padding, rc_client.bottom + u_padding,
        winbase::MulDiv(EXTERNAL_CONTENT_WIDTH96, u_dpi as i32, 96),
        winbase::MulDiv(EXTERNAL_CONTENT_HEIGHT96, u_dpi as i32, 96),
        winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE); }

    // Send a new font to all child controls (the 'plugin' content is subclassed to ignore WM_SETFONT)
    let h_font_old = get_window_font(h_wnd_static);
    let mut lf_text = wingdi::LOGFONTW::default();
    unsafe { winuser::SystemParametersInfoForDpi(winuser::SPI_GETICONTITLELOGFONT, mem::size_of::<wingdi::LOGFONTW>() as u32, &mut lf_text as *mut wingdi::LOGFONTW as *mut c_void, 0, u_dpi); }
    let h_font_new = unsafe { wingdi::CreateFontIndirectW(&lf_text) };
    if h_font_new != ptr::null_mut()
    {
        unsafe { wingdi::DeleteObject(h_font_old as *mut c_void); }
        enum_child_windows(h_wnd, move |h_wnd, _l_param|
        {
            unsafe { winuser::SendMessageW(h_wnd, winuser::WM_SETFONT, h_font_new as usize, make_l_param(TRUE as u16, 0) as isize); }
            return TRUE;
        });
    }
}

#[no_mangle]
pub extern "C" fn do_initial_window_setup(h_wnd: HWND) -> LRESULT
{
    let h_inst = get_hinstance_for_h_wnd(h_wnd);
    // Resize the window to account for DPI. The window might have been created
    // on a monitor that has > 96 DPI. Windows does not send a window a DPI change
    // when it is created, even if it is created on a monitor with a DPI > 96
    let mut rc_window = RECT::default();

    // Determine the DPI to use, according to the DPI awareness mode
	let _dpi_awareness = get_thread_dpi_awareness();

	let u_dpi = get_maybe_dpi_by_awareness(h_wnd).unwrap_or(96);

    unsafe { winuser::GetWindowRect(h_wnd, &mut rc_window); }
    rc_window.right = rc_window.left + unsafe { winbase::MulDiv(WINDOW_WIDTH96, u_dpi as i32, 96) };
    rc_window.bottom = rc_window.top + unsafe { winbase::MulDiv(WINDOW_HEIGHT96, u_dpi as i32, 96) };
    unsafe {
        winuser::SetWindowPos(h_wnd, 
            ptr::null_mut(), 
            rc_window.right, 
            rc_window.top, 
            rc_window.right - rc_window.left, 
            rc_window.bottom - rc_window.top, 
            winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE);
    }

    // Create a static control for use displaying DPI-related information.
    // Initially the static control will not be sized, but we will next DPI
    // scale it with a helper function.
    let static_str = to_wstring("STATIC");
    let h_wnd_name_static = to_wstring(HWND_NAME_STATIC);
    let h_wnd_static = unsafe {
        winuser::CreateWindowExW(
            winuser::WS_EX_LEFT, 
            static_str.as_ptr(), 
            h_wnd_name_static.as_ptr(), 
            winuser::SS_LEFT | winuser::WS_CHILD | winuser::WS_VISIBLE,
            0, 0, 0, 0, h_wnd, ptr::null_mut(), h_inst, ptr::null_mut())
    };
    if h_wnd_static == ptr::null_mut()
    {
        return -1;
    }

    // Create some buttons
    let button_str = to_wstring("BUTTON");
    let h_wnd_name_checkbox = to_wstring(HWND_NAME_CHECKBOX);
    let _h_wnd_checkbox = unsafe { winuser::CreateWindowExW(0, button_str.as_ptr(), h_wnd_name_checkbox.as_ptr(), 
        winuser::WS_TABSTOP | winuser::WS_VISIBLE | winuser::WS_CHILD | winuser::BS_DEFPUSHBUTTON | winuser::BS_CHECKBOX, 
        0, 0, 0, 0, h_wnd, ptr::null_mut(), h_inst, ptr::null_mut()) };
    let h_wnd_name_radio = to_wstring(HWND_NAME_RADIO);
    let _h_wnd_radio = unsafe { winuser::CreateWindowExW(0, button_str.as_ptr(), h_wnd_name_radio.as_ptr(),
        winuser::BS_PUSHBUTTON | winuser::BS_TEXT | winuser::BS_DEFPUSHBUTTON | winuser::BS_USERBUTTON | winuser::BS_AUTORADIOBUTTON | winuser::WS_CHILD | winuser::WS_OVERLAPPED | winuser::WS_VISIBLE,
        0, 0, 0, 0, h_wnd, ptr::null_mut(), h_inst, ptr::null_mut()) };
    let h_wnd_name_dialog = to_wstring(HWND_NAME_DIALOG);
    let _h_wnd_dialog = unsafe {  winuser::CreateWindowExW(0, button_str.as_ptr(), h_wnd_name_dialog.as_ptr(),
        winuser::WS_TABSTOP | winuser::WS_VISIBLE | winuser::WS_CHILD | winuser::BS_DEFPUSHBUTTON,
        0, 0, 0, 0, h_wnd, IDM_SHOWDIALOG as HMENU, h_inst, ptr::null_mut()) };

    // Load an HWND from an external source (a DLL in this example)
    //
    // HWNDs from external sources might not support Per-Monitor V2 awareness. Hosting HWNDs that
    // don't support the same DPI awareness mode as their host can lead to rendering problems.
    // When child-HWND DPI isolation is enabled, Windows will try to let that HWND run in its native
    // DPI scaling mode (which might or might not have been defined explicitly). 

    // First, determine if we are in the correct mode to use this feature
    let prop_dpi_isolation = to_wstring(PROP_DPIISOLATION);
    let b_dpi_isolation: BOOL = unsafe { winuser::GetPropW(h_wnd, prop_dpi_isolation.as_ptr()) as BOOL };

    let mut previous_dpi_context = ptr::null_mut();
	let mut previous_dpi_hosting_behavior = DPI_HOSTING_BEHAVIOR::default();
    if b_dpi_isolation == TRUE
    {
        previous_dpi_hosting_behavior = set_thread_dpi_hosting_behavior(DPI_HOSTING_BEHAVIOR_MIXED);

        // For this example, we'll have the external content run with System-DPI awareness
		previous_dpi_context = set_thread_dpi_awareness_context(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE);
    }

    let h_wnd_external = unsafe {
         dll_plugin_import::create_content_hwnd(h_inst, EXTERNAL_CONTENT_WIDTH96, EXTERNAL_CONTENT_HEIGHT96)
    };

    // Return the thread context and hosting behavior to its previous value, if using DPI-isolation
	if b_dpi_isolation == TRUE
	{
		set_thread_dpi_awareness_context(previous_dpi_context);
		set_thread_dpi_hosting_behavior(previous_dpi_hosting_behavior);
	}

    // After the external content HWND was create with a system-DPI awareness context, reparent it
    let _h_wnd_result = unsafe { winuser::SetParent(h_wnd_external, h_wnd) };

    // DPI scale child-windows
    //UpdateAndDpiScaleChildWindows(h_wnd, u_dpi);
	update_and_dpi_scale_child_windows(h_wnd, u_dpi);

    return 0;
}

// DPI Change handler. on WM_DPICHANGE resize the window and
// then call a function to redo layout for the child controls
#[no_mangle]
pub extern "C" fn handle_dpi_change(h_wnd: HWND, w_param: WPARAM, l_param: LPARAM) -> LRESULT
{
    let static_str = to_wstring("STATIC");
    let h_wnd_static = unsafe { winuser::FindWindowExW(h_wnd, ptr::null_mut(), static_str.as_ptr(), ptr::null_mut()) };

    if h_wnd_static != ptr::null_mut()
    {
        let u_dpi = HIWORD(w_param as u32) as UINT;

        // Resize the window
        let lprc_new_scale: *const RECT = l_param as *const RECT;
        let new_scale = unsafe { &*lprc_new_scale };

        unsafe {
            winuser::SetWindowPos(h_wnd, ptr::null_mut(), new_scale.left, new_scale.top,
                new_scale.right - new_scale.left, new_scale.bottom - new_scale.top,
                winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE);
        }

        // Redo layout of the child controls
		update_and_dpi_scale_child_windows(h_wnd, u_dpi);
    }

    return 0;
}

// Create the sample window and set its initial size, based off of the
// DPI awareness mode that it's running under
#[no_mangle]
pub extern "C" fn create_sample_window(h_wnd_dlg: HWND,
    context: DPI_AWARENESS_CONTEXT, 
    b_enable_non_client_dpi_scaling: BOOL, 
    b_child_window_dpi_isolation: BOOL)
{
    let h_inst = get_hinstance_for_h_wnd(h_wnd_dlg);

    // Store the current thread's DPI-awareness context
    let previous_dpi_context = set_thread_dpi_awareness_context(context);

    // Create the window. Initially create it using unscaled (96 DPI)
    // sizes. We'll resize the window after it's created

    let mut create_params = CreateParams {
        b_enable_non_client_dpi_scaling: b_enable_non_client_dpi_scaling,
        b_child_window_dpi_isolation: b_child_window_dpi_isolation,
    };

    // Windows 10 (1803) supports child-HWND DPI-mode isolation. This enables
    // child HWNDs to run in DPI-scaling modes that are isolated from that of 
    // their parent (or host) HWND. Without child-HWND DPI isolation, all HWNDs 
    // in an HWND tree must have the same DPI-scaling mode.
	let previous_dpi_hosting_behavior = if b_child_window_dpi_isolation == TRUE
	{
		set_thread_dpi_hosting_behavior(DPI_HOSTING_BEHAVIOR_MIXED)
	} else {
        DPI_HOSTING_BEHAVIOR::default()
    };

    let h_menu = unsafe { winuser::LoadMenuW(h_inst, IDC_MAINMENU as *mut wchar_t) };

    let empty_str = to_wstring("");
    let class_name_str = to_wstring(WINDOWCLASSNAME);
    let h_wnd = unsafe { winuser::CreateWindowExW(0, class_name_str.as_ptr(), empty_str.as_ptr(), 
        winuser::WS_OVERLAPPEDWINDOW | winuser::WS_HSCROLL | winuser::WS_VSCROLL,
        winuser::CW_USEDEFAULT, 0, WINDOW_WIDTH96, WINDOW_HEIGHT96, h_wnd_dlg, h_menu,
        h_inst, &mut create_params as *mut CreateParams as *mut c_void) };

    unsafe { winuser::ShowWindow(h_wnd, winuser::SW_SHOWNORMAL) };

    // Restore the current thread's DPI awareness context
    set_thread_dpi_awareness_context(previous_dpi_context);

	// Restore the current thread DPI hosting behavior, if we changed it.
	if b_child_window_dpi_isolation == TRUE
	{
		set_thread_dpi_hosting_behavior(previous_dpi_hosting_behavior);
	}
}

// The dialog procedure for the sample host window
#[no_mangle]
pub extern "system" fn host_dialog_proc(h_wnd_dlg: HWND, message: UINT, w_param: WPARAM, _l_param: LPARAM) -> LRESULT
{
    match message {
        winuser::WM_CTLCOLORDLG => {
            return get_stock_brush(wingdi::WHITE_BRUSH) as LRESULT;
        },
        winuser::WM_CTLCOLORSTATIC => {
            return get_stock_brush(wingdi::WHITE_BRUSH) as LRESULT;
        },
        winuser::WM_INITDIALOG => { 
            let app_description = to_wstring(
    r#"This sample app lets you create windows with different DPI Awareness modes so
    that you can observe how Win32 windows behave under these modes.
    Each window will show different behaviors depending on the mode (will be blurry or
    crisp, non-client area will scale differently, etc.).
    \r\n\r\n
    The best way to observe these differences is to move each window to a display with a
    different display scaling (DPI) value. On single-display devices you can simulate
    this by changing the display scaling value of your display (the "Change the size
    of text, apps, and other items" setting in the Display settings page of the Settings
    app, as of Windows 10, 1703). Make these settings changes while the app is still
    running to observe the different DPI-scaling behavior.
    "#
            );
            unsafe { winuser::SetDlgItemTextW(h_wnd_dlg, IDC_EDIT1, app_description.as_ptr()); }
            return 0;
        },
        winuser::WM_COMMAND => {
            let mut context: DPI_AWARENESS_CONTEXT = ptr::null_mut();
            let mut b_non_client_scaling = FALSE;
            let mut b_child_window_dpi_isolation = FALSE;
            match LOWORD(w_param as u32) as i32 {
                IDC_BUTTON_UNAWARE => {
                    context = DPI_AWARENESS_CONTEXT_UNAWARE;
                },
                IDC_BUTTON_SYSTEM => {
                    context = DPI_AWARENESS_CONTEXT_SYSTEM_AWARE;
                },
                IDC_BUTTON_81 => {
                    context = DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE;
                },
                IDC_BUTTON_1607 => {
                    context = DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE;
                    b_non_client_scaling = TRUE;
                },
                IDC_BUTTON_1703 => {
                    context = DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
                },
                IDC_BUTTON_1803 => {
                    context = DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
                    b_child_window_dpi_isolation = TRUE;
                },
                IDM_EXIT => {
                    unsafe { winuser::DestroyWindow(h_wnd_dlg); }
                    return 0;
                },
                _ => {}
            }

            if context != ptr::null_mut()
            {
                create_sample_window(h_wnd_dlg, context, b_non_client_scaling, b_child_window_dpi_isolation);
            }
            return TRUE as LRESULT;

        },
        winuser::WM_CLOSE => {
            unsafe { winuser::DestroyWindow(h_wnd_dlg); }
            return 0;
        },
        winuser::WM_DESTROY => {
            delete_window_font(h_wnd_dlg);
            unsafe  { winuser::PostQuitMessage(0); }
            return FALSE as LRESULT;
        },
        _ => {
        }
    }

    return FALSE as LRESULT;
}

// The window procedure for the sample windows
#[no_mangle]
pub extern "system" fn wnd_proc(h_wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT
{
    match message {
        winuser::WM_NCCREATE => {
            // Enable per-monitor DPI scaling for caption, menu, and top-level
            // scroll bars.
            //
            // Non-client area (scroll bars, caption bar, etc.) does not DPI scale
            // automatically on Windows 8.1. In Windows 10 (1607) support was added
            // for this via a call to EnableNonClientDpiScaling. Windows 10 (1703)
            // supports this automatically when the DPI_AWARENESS_CONTEXT is
            // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2.
            //
            // Here we are detecting if a BOOL was set to enable non-client DPI scaling
            // via the call to CreateWindow that resulted in this window. Doing this
            // detection is only necessary in the context of this sample.
            let create_struct = unsafe { &*(l_param as *const winuser::CREATESTRUCTA) };
            let create_params = unsafe { &*(create_struct.lpCreateParams as *const CreateParams) };

            if create_params.b_enable_non_client_dpi_scaling == TRUE
            {
                unsafe { winuser::EnableNonClientDpiScaling(h_wnd); }
            }

            // Store a flag on the window to note that it'll run its child in a different awareness
			if create_params.b_child_window_dpi_isolation == TRUE
			{ 
                let prop_dpi_isolation_str = to_wstring(PROP_DPIISOLATION);
				unsafe { winuser::SetPropW(h_wnd, prop_dpi_isolation_str.as_ptr(), TRUE as HANDLE) };
			}

            return unsafe { winuser::DefWindowProcW(h_wnd, message, w_param, l_param) };
        },

        // Set static text background to white.
        winuser::WM_CTLCOLORSTATIC => {
            return get_stock_brush(wingdi::WHITE_BRUSH) as LRESULT;
        },

        winuser::WM_CREATE => {
            return do_initial_window_setup(h_wnd);
        },

        // On DPI change resize the window, scale the font, and update
        // the DPI-info string
        winuser::WM_DPICHANGED => {
            return handle_dpi_change(h_wnd, w_param, l_param);
        },

        winuser::WM_CLOSE => {
            unsafe { winuser::DestroyWindow(h_wnd); }
            return 0;
        },

        winuser::WM_COMMAND => {
            let wm_id = LOWORD(w_param as u32) as i32;
            // Parse the menu selections:
            match wm_id{
                res::IDM_SHOWDIALOG => {
                    show_file_open_dialog(h_wnd);
                    return 0;
                },

                _ => {
                    return unsafe { winuser::DefWindowProcW(h_wnd, message, w_param, l_param) };
                }
            }
        },

        winuser::WM_DESTROY => {
            delete_window_font(h_wnd);

            return 0;
        },

        _ => {
            return unsafe { winuser::DefWindowProcW(h_wnd, message, w_param, l_param) };
        }
    }
}

pub fn main() {
/*#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn wWinMain(h_inst: HINSTANCE, _h_inst_2: HINSTANCE, _str: LPWSTR, n_cmd_show: INT) -> INT
{*/
    //show_console_window();

    let mut wcex = winuser::WNDCLASSEXW::default();
    let wcex_classname = to_wstring(WINDOWCLASSNAME);
    let h_inst = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };
    let n_cmd_show = winuser::SW_SHOWNORMAL;

    wcex.cbSize = mem::size_of::<winuser::WNDCLASSEXW>() as u32;
    wcex.style = winuser::CS_HREDRAW | winuser::CS_VREDRAW;
	wcex.lpfnWndProc = Some(wnd_proc);
    wcex.hInstance = h_inst;
    wcex.hCursor = unsafe { winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW) };
    wcex.hbrBackground = (winuser::COLOR_WINDOW + 1) as HBRUSH;
    wcex.lpszClassName = wcex_classname.as_ptr();

    unsafe { winuser::RegisterClassExW(&wcex); }

    //g_hInst = hInstance; // Store instance handle in our global variable

    // Create the host window
    let h_host_dlg = unsafe { winuser::CreateDialogParamW(h_inst, IDD_DIALOG1 as LPCWSTR, ptr::null_mut(), Some(host_dialog_proc), 0) };
    if h_host_dlg == ptr::null_mut()
    {
        std::process::exit(0);
    }

    unsafe { winuser::ShowWindow(h_host_dlg, n_cmd_show); }

    let mut msg = winuser::MSG::default();

    // Main message loop:
    while unsafe { winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) == TRUE }
    {
        unsafe { winuser::TranslateMessage(&msg) };
        unsafe { winuser::DispatchMessageW(&msg) };
    }

    std::process::exit(msg.wParam as i32);
}

fn show_console_window() {
    unsafe {consoleapi::AllocConsole()};
}

fn hide_console_window() {
    let window = unsafe {wincon::GetConsoleWindow()};
    // https://msdn.microsoft.com/en-us/library/windows/desktop/ms633548%28v=vs.85%29.aspx
    if window != ptr::null_mut() {
        unsafe {
            winuser::ShowWindow(window, winuser::SW_HIDE);
        }
    }
}

#[no_mangle]
pub extern "C" fn show_file_open_dialog(h_wnd: HWND)
{
    let sz_file = &mut [0;MAX_PATH];
    let filter_str = to_wstring("All\0*.*\0Text\0*.TXT\0");

    let mut ofn = commdlg::OPENFILENAMEW {
        hwndOwner: h_wnd,
        lpstrFile: sz_file.as_mut_ptr(),
        nMaxFile: sz_file.len() as u32,
        lpstrFilter: filter_str.as_ptr(),
        nFilterIndex: 1,
        Flags: commdlg::OFN_PATHMUSTEXIST | commdlg::OFN_FILEMUSTEXIST,

        ..Default::default()
    };

    // Display the Open dialog box.
    unsafe { commdlg::GetOpenFileNameW(&mut ofn); }
}

// Find the child static control, get the font for the control, then
// delete it
#[no_mangle]
pub extern "C" fn delete_window_font(h_wnd: HWND)
{
    let h_wnd_static = unsafe { winuser::GetWindow(h_wnd, winuser::GW_CHILD) };
    if h_wnd_static == ptr::null_mut()
    {
        return;
    }

    // Get a handle to the font
    let h_font = get_window_font(h_wnd_static);
    if h_font == ptr::null_mut()
    {
        return;
    }

    set_window_font(h_wnd_static, ptr::null_mut(), FALSE);
    unsafe { wingdi::DeleteObject(h_font as *mut c_void) };
}

pub struct EnumChildWindowProcContext {
    func: Box<dyn EnumChildWindowProcCallback>
}

pub trait EnumChildWindowProcCallback {
    fn call(&self, h_wnd: HWND, l_param: LPARAM) -> BOOL;
}

impl EnumChildWindowProcCallback for EnumChildWindowProcContext {
    fn call(&self, h_wnd: HWND, l_param: LPARAM) -> BOOL {
        self.func.call(h_wnd, l_param)
    }
}

impl <F> EnumChildWindowProcCallback for F where F: Fn(HWND, LPARAM) -> BOOL {
    fn call(&self, h_wnd: HWND, l_param: LPARAM) -> BOOL {
        self(h_wnd, l_param)
    }
}

#[no_mangle]
pub extern "system" fn enum_child_windows_proc(h_wnd: HWND, l_param: LPARAM) -> BOOL {
    let v = l_param as *mut EnumChildWindowProcContext;

    let result = unsafe { v.as_ref() }.unwrap().call(h_wnd, l_param);

    result
}

pub fn enum_child_windows<F>(h_wnd: HWND, f: F) where F: EnumChildWindowProcCallback + 'static {
    let callback = Box::new(EnumChildWindowProcContext { func: Box::new(f) });
    unsafe { winuser::EnumChildWindows(h_wnd, Some(enum_child_windows_proc), Box::into_raw(callback) as isize); }
}
