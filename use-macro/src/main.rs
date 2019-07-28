use declare_macro::declare_functions;

pub type BOOL = i32;

declare_functions! {
	extern "system" {
		#[library(USER32)]
		pub fn IsProcessDPIAware() -> BOOL;
		#[library(USER32)]
		pub fn SetProcessDPIAware() -> BOOL;
	}
}

fn main() {
	if let Some(is_process_dpi_aware) = dynamic::IsProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { is_process_dpi_aware() });
	}
	if let Some(set_process_dpi_aware) = dynamic::SetProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { set_process_dpi_aware() });
	}
	if let Some(is_process_dpi_aware) = dynamic::IsProcessDPIAware.as_ref() {
		println!("{:?}", unsafe { is_process_dpi_aware() });
	}
}
