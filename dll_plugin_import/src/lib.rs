use winapi::{
    shared::{
        minwindef::{
            INT,
            HINSTANCE,
        },
        windef::{
            HWND,
        },
    },
};

#[link(name = "dll_plugin.dll")]
extern "C" {
    pub fn create_content_hwnd(h_instance: HINSTANCE, n_width: INT, n_height: INT) -> HWND;
}
