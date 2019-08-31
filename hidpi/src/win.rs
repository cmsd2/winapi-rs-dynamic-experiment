use declare_macro::declare_functions;
use std::io;

use winapi::{
	shared::{
        minwindef::BOOL, ntdef::HANDLE, winerror::S_OK,
        windef::{
            DPI_AWARENESS,
            DPI_AWARENESS_CONTEXT,
            DPI_HOSTING_BEHAVIOR,
        }
    },
	um::{
        shellscalingapi::PROCESS_DPI_AWARENESS,
        shellscalingapi::PROCESS_DPI_UNAWARE,
        shellscalingapi::PROCESS_SYSTEM_DPI_AWARE,
        shellscalingapi::PROCESS_PER_MONITOR_DPI_AWARE,
        winnt::HRESULT
    },
};

lazy_static::lazy_static! {
	static ref USER32: Option<libloading::Library> = { libloading::Library::new("user32.dll").ok() };
	static ref SHCORE: Option<libloading::Library> = { libloading::Library::new("shcore.dll").ok() };
}

declare_functions! {
	extern "system" {
		#[library(USER32)]
		pub fn IsProcessDPIAware() -> BOOL;
		#[library(USER32)]
		pub fn SetProcessDPIAware() -> BOOL;
		#[library(SHCORE)]
		pub fn GetProcessDpiAwareness(
			hProcess: HANDLE,
			value: *mut PROCESS_DPI_AWARENESS,
		) -> HRESULT;
        #[library(SHCORE)]
        pub fn SetProcessDpiAwareness(
            value: PROCESS_DPI_AWARENESS
        ) -> HRESULT;
        #[library(USER32)]
        pub fn GetThreadDpiAwarenessContext() -> DPI_AWARENESS_CONTEXT;
        #[library(USER32)]
        pub fn GetAwarenessFromDpiAwarenessContext(
            context: DPI_AWARENESS_CONTEXT
        ) -> DPI_AWARENESS;
        #[library(USER32)]
        pub fn AreDpiAwarenessContextsEqual(
            a: DPI_AWARENESS_CONTEXT,
            b: DPI_AWARENESS_CONTEXT
        ) -> BOOL;
        #[library(USER32)]
        pub fn SetThreadDpiHostingBehavior(
            b: DPI_HOSTING_BEHAVIOR
        ) -> DPI_HOSTING_BEHAVIOR;
        #[library(USER32)]
        pub fn GetThreadDpiHostingBehavior() -> DPI_HOSTING_BEHAVIOR;
        #[library(USER32)]
        pub fn SetThreadDpiAwarenessContext(
            dpiContext: DPI_AWARENESS_CONTEXT
        ) -> DPI_AWARENESS_CONTEXT;
	}
}

pub enum WinDpiAwareness {
    ProcessDpiUnaware,
    ProcessSystemDpiAware,
    ProcessPerMonitorDpiAware,
    Unknown(PROCESS_DPI_AWARENESS),
}

impl From<PROCESS_DPI_AWARENESS> for WinDpiAwareness {
    fn from(a: PROCESS_DPI_AWARENESS) -> WinDpiAwareness {
        match a {
            PROCESS_DPI_UNAWARE => WinDpiAwareness::ProcessDpiUnaware,
            PROCESS_SYSTEM_DPI_AWARE => WinDpiAwareness::ProcessSystemDpiAware,
            PROCESS_PER_MONITOR_DPI_AWARE => WinDpiAwareness::ProcessPerMonitorDpiAware,
            other => WinDpiAwareness::Unknown(other),
        }
    }
}

pub fn are_dpi_awareness_contexts_equal(a: DPI_AWARENESS_CONTEXT, b: DPI_AWARENESS_CONTEXT) -> Option<BOOL> { 
    if let Some(are_dpi_awareness_contexts_equal) = dynamic::AreDpiAwarenessContextsEqual.as_ref() {
        Some(unsafe { are_dpi_awareness_contexts_equal(a, b) })
    } else {
        None
    }
}

pub fn get_awareness_from_dpi_awareness_context(context: DPI_AWARENESS_CONTEXT) -> Option<DPI_AWARENESS> {
    if let Some(get_awareness_from_dpi_awareness_context) = dynamic::GetAwarenessFromDpiAwarenessContext.as_ref() {
        Some(unsafe  { get_awareness_from_dpi_awareness_context(context) })
    } else {
        None
    }
}

pub fn get_thread_dpi_awareness_context() -> Option<DPI_AWARENESS_CONTEXT> {
    if let Some(get_thread_dpi_awareness_context) = dynamic::GetThreadDpiAwarenessContext.as_ref() {
        Some(unsafe { get_thread_dpi_awareness_context() })
    } else {
        None
    }
}

pub fn is_process_dpi_aware() -> Option<bool> {
    if let Some(is_process_dpi_aware) = dynamic::IsProcessDPIAware.as_ref() {
        Some(unsafe { is_process_dpi_aware() } != 0)
    } else {
        None
    }
}

pub fn get_process_dpi_awareness() -> io::Result<Option<WinDpiAwareness>> {
    if let Some(get_process_dpi_awareness) = dynamic::GetProcessDpiAwareness.as_ref() {
		let mut awareness: PROCESS_DPI_AWARENESS = 0;
		if S_OK == unsafe { get_process_dpi_awareness(std::ptr::null_mut(), &mut awareness) } {
            Ok(Some(From::from(awareness)))
		} else {
            Err(io::Error::last_os_error())
        }
	} else {
        Ok(None)
    }
}

pub fn set_process_dpi_aware() -> Option<bool> {
    if let Some(set_process_dpi_aware) = dynamic::SetProcessDPIAware.as_ref() {
        Some(unsafe { set_process_dpi_aware() } != 0)
    } else { 
        None
    }
}

pub fn set_process_dpi_awareness(win_awareness: WinDpiAwareness) -> io::Result<bool> {
    let awareness = match win_awareness {
        WinDpiAwareness::ProcessDpiUnaware => PROCESS_DPI_UNAWARE,
        WinDpiAwareness::ProcessSystemDpiAware => PROCESS_SYSTEM_DPI_AWARE,
        WinDpiAwareness::ProcessPerMonitorDpiAware => PROCESS_PER_MONITOR_DPI_AWARE,
        WinDpiAwareness::Unknown(o) => o,
    };

    if let Some(set_process_dpi_awareness) = dynamic::SetProcessDpiAwareness.as_ref() {
        if S_OK == unsafe { set_process_dpi_awareness(awareness) } {
            Ok(true)
        } else {
            Err(io::Error::last_os_error())
        }
    } else {
        Ok(false)
    }
}

pub fn set_thread_dpi_awareness_context(context: DPI_AWARENESS_CONTEXT) -> Option<DPI_AWARENESS_CONTEXT> {
    if let Some(set_thread_dpi_awareness_context) = dynamic::SetThreadDpiAwarenessContext.as_ref() {
        Some(unsafe { set_thread_dpi_awareness_context(context) })
    } else {
        None
    }
}

pub fn get_thread_dpi_hosting_behavior() -> Option<DPI_HOSTING_BEHAVIOR>  {
    if let Some(get_thread_dpi_hosting_behavior) = dynamic::GetThreadDpiHostingBehavior.as_ref() {
        Some(unsafe { get_thread_dpi_hosting_behavior() })
    } else {
        None
    }
}

pub fn set_thread_dpi_hosting_behavior(behavior: DPI_HOSTING_BEHAVIOR) -> Option<DPI_HOSTING_BEHAVIOR>  {
    if let Some(set_thread_dpi_hosting_behavior) = dynamic::SetThreadDpiHostingBehavior.as_ref() {
        Some(unsafe { set_thread_dpi_hosting_behavior(behavior) })
    } else {
        None
    }
}