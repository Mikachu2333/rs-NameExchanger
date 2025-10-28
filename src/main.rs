//#![windows_subsystem = "windows"]

use winsafe::{
    self as w,
    co::{WM, WS, WS_EX},
    gui::{self, Icon},
    prelude::*,
};

fn main() {
    if let Err(e) = ExchangerWindow::create_and_run() {
        eprintln!("{}", e);
    }
}

#[derive(Clone)]
pub struct ExchangerWindow {
    wnd: gui::WindowMain,
    btn_exchange: gui::Button,
    label_path1: gui::Label,
    label_path2: gui::Label,
    edit_path1: gui::Edit,
    edit_path2: gui::Edit,
}

impl ExchangerWindow {
    pub fn create_and_run() -> w::AnyResult<i32> {
        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "Name Exchanger",
            size: gui::dpi(400, 300),
            class_name: "NE_CLASS",
            style: WS::BORDER | WS::POPUP,
            ex_style: WS_EX::ACCEPTFILES | WS_EX::CONTEXTHELP | WS_EX::TOOLWINDOW,
            class_icon: Icon::None,
            ..Default::default()
        });

        let label_path1 = gui::Label::new(
            &wnd,
            gui::LabelOpts {
                text: "Path 1",
                position: gui::dpi(15, 57),
                size: gui::dpi(57, 23),
                ..Default::default()
            },
        );

        let label_path2 = gui::Label::new(
            &wnd,
            gui::LabelOpts {
                text: "Path 2",
                position: gui::dpi(15, 137),
                size: gui::dpi(57, 23),
                ..Default::default()
            },
        );

        let edit_path1 = gui::Edit::new(
            &wnd,
            gui::EditOpts {
                position: gui::dpi(16, 80),
                width: 368,
                height: 40,
                ..Default::default()
            },
        );

        let edit_path2 = gui::Edit::new(
            &wnd,
            gui::EditOpts {
                position: gui::dpi(16, 160),
                width: 368,
                height: 40,
                ..Default::default()
            },
        );

        let btn_exchange = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "Exchange",
                height: 59,
                width: 58,
                position: gui::dpi(142, 223),
                ..Default::default()
            },
        );

        let new_self = Self {
            wnd,
            btn_exchange,
            label_path1,
            label_path2,
            edit_path1,
            edit_path2,
        };
        new_self.events();

        new_self.wnd.run_main(None)
    }

    fn events(&self) {
        let wnd = self.wnd.clone();

        self.wnd.on().wm(WM::DROPFILES, {
            move |p: winsafe::msg::WndMsg| -> w::AnyResult<isize> {
                //效果太差，太多函数缺失，需要手动调用，评价为不如cpp
                dbg!(p.msg_id,p.wparam);
                panic!();
            }
        });

        self.btn_exchange.on().bn_clicked(move || {
            //wnd.hwnd().SetWindowText("Hello, world!")?;
            panic!();
        });
    }
}
