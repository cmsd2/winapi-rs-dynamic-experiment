use declare_macro::declare_functions;

use winapi::{
	shared::{minwindef::BOOL, ntdef::HANDLE, winerror::S_OK},
	um::{shellscalingapi::PROCESS_DPI_AWARENESS, winnt::HRESULT},
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
	}
}

fn main() {
	if let Some(is_process_dpi_aware) = dynamic::IsProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { is_process_dpi_aware() });
	}
	if let Some(get_process_dpi_awareness) = dynamic::GetProcessDpiAwareness.as_ref() {
		let mut awareness: PROCESS_DPI_AWARENESS = 0;
		if S_OK == unsafe { get_process_dpi_awareness(std::ptr::null_mut(), &mut awareness) } {
			println!("{}", awareness);
		}
	}
	if let Some(set_process_dpi_aware) = dynamic::SetProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { set_process_dpi_aware() });
	}
	if let Some(is_process_dpi_aware) = dynamic::IsProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { is_process_dpi_aware() });
	}
	if let Some(get_process_dpi_awareness) = dynamic::GetProcessDpiAwareness.as_ref() {
		let mut awareness: PROCESS_DPI_AWARENESS = 0;
		if S_OK == unsafe { get_process_dpi_awareness(std::ptr::null_mut(), &mut awareness) } {
			println!("{}", awareness);
		}
	}
}
