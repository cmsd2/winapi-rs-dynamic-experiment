use hidpi::{
	is_process_dpi_aware,
	set_process_dpi_aware,
	set_process_dpi_awareness,
	get_process_dpi_awareness,
	DpiAwareness,
};

fn main() {
	println!("is_process_dpi_aware? {:?}", is_process_dpi_aware());
	print_awareness();
	println!("set_process_dpi_aware? {:?}", set_process_dpi_aware());
	println!("is_process_dpi_aware? {:?}", is_process_dpi_aware());
	print_awareness();
	println!("set_process_dpi_awareness {:?}", set_process_dpi_awareness(DpiAwareness::Unaware));
	print_awareness();
	println!("set_process_dpi_awareness {:?}", set_process_dpi_awareness(DpiAwareness::System));
	print_awareness();
	println!("set_process_dpi_awareness {:?}", set_process_dpi_awareness(DpiAwareness::PerMonitor));
	print_awareness();
}

fn print_awareness() {
	match get_process_dpi_awareness() {
		Ok(awareness) => println!("get_process_dpi_awareness: {:?}", awareness),
		Err(e) => println!("get_process_dpi_awareness error: {:?}", e),
	}
}