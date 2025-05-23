#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

use core::ffi::c_void;
use exchange_lib::exchange;
use mslnk::ShellLink;
use std::path::PathBuf;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

enum TrayMessage {
    Right,
    Left
}


#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (300, 115), position: (400, 400), title: "Basic example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] )]
    window: nwg::Window,

    #[nwg_control(text: "Heisenberg", size: (280, 25), position: (10, 10))]
    name_edit: nwg::TextInput,

    #[nwg_control(text: "Say my name", size: (280, 60), position: (10, 40))]
    #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    hello_button: nwg::Button
}

impl BasicApp {

    fn say_hello(&self) {
        nwg::simple_message("Hello", &format!("Hello {}", self.name_edit.text()));
    }
    
    fn say_goodbye(&self) {
        nwg::simple_message("Goodbye", &format!("Goodbye {}", self.name_edit.text()));
        nwg::stop_thread_dispatch();
    }

}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}