#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

mod msgbox {
    include!("../lib/msgbox.rs");
}

use name_exchanger_lib::exchange_rs;
use native_windows_gui as nwg;
use nwg::NativeUi;

const WS_MAXIMIZEBOX: u32 = 0x00010000;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WM_DROPFILES: u32 = 0x0233;
const WM_GETMINMAXINFO: u32 = 0x0024;
const MAX_PATH: usize = 260;
const MIN_WIDTH: i32 = 450;
const MIN_HEIGHT: i32 = 250;

static mut OLD_WND_PROC: Option<isize> = None;

pub struct App {
    window: nwg::Window,
    font: nwg::Font,

    label_path1: nwg::Label,
    text_path1: nwg::TextInput,

    label_path2: nwg::Label,
    text_path2: nwg::TextInput,

    btn_exchange: nwg::Button,
    layout: nwg::FlexboxLayout,

    path1: RefCell<Option<PathBuf>>,
    path2: RefCell<Option<PathBuf>>,
}

pub struct AppUi {
    inner: Rc<App>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl App {
    fn on_exchange(&self) {
        let p1_str = self.text_path1.text();
        let p2_str = self.text_path2.text();

        if p1_str.is_empty() || p2_str.is_empty() {
            msgbox::warn_msgbox("请输入连个完整的文件或文件夹路径！", "提示", 0);
            return;
        }

        let p1 = PathBuf::from(p1_str);
        let p2 = PathBuf::from(p2_str);

        match exchange_rs(&p1, &p2) {
            Ok(()) => {
                msgbox::info_msgbox("名称交换成功！", "成功", 0);
                self.text_path1.set_text("");
                self.text_path2.set_text("");
                *self.path1.borrow_mut() = None;
                *self.path2.borrow_mut() = None;
            }
            Err(e) => {
                msgbox::error_msgbox(format!("交换失败: {}", e), "错误", 0);
            }
        }
    }

    fn drop_files(&self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }

        if paths.len() == 1 {
            if self.path1.borrow().is_none() && self.text_path1.text().is_empty() {
                self.text_path1.set_text(&paths[0].to_string_lossy());
                *self.path1.borrow_mut() = Some(paths[0].clone());
            } else {
                self.text_path2.set_text(&paths[0].to_string_lossy());
                *self.path2.borrow_mut() = Some(paths[0].clone());
            }
        } else {
            self.text_path1.set_text(&paths[0].to_string_lossy());
            *self.path1.borrow_mut() = Some(paths[0].clone());

            self.text_path2.set_text(&paths[1].to_string_lossy());
            *self.path2.borrow_mut() = Some(paths[1].clone());
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.window.handle.destroy();
    }
}

impl nwg::NativeUi<AppUi> for App {
    fn build_ui(mut data: App) -> Result<AppUi, nwg::NwgError> {
        use nwg::stretch::{
            geometry::{Rect, Size},
            style::{Dimension as D, FlexDirection},
        };
        use nwg::Event as E;

        const MARGIN: Rect<D> = Rect {
            start: D::Points(5.0),
            end: D::Points(5.0),
            top: D::Points(5.0),
            bottom: D::Points(5.0),
        };
        const PADDING: Rect<D> = Rect {
            start: D::Points(10.0),
            end: D::Points(10.0),
            top: D::Points(10.0),
            bottom: D::Points(10.0),
        };

        nwg::Window::builder()
            .size((MIN_WIDTH, MIN_HEIGHT))
            .position((0, 0))
            .title("名称交换器")
            .flags(
                nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE | nwg::WindowFlags::RESIZABLE,
            )
            .build(&mut data.window)?;

        nwg::Font::builder()
            .family("Microsoft YaHei UI")
            .size(20)
            .weight(400)
            .build(&mut data.font)?;

        nwg::Label::builder()
            .text("文件/文件夹 1:")
            .parent(&data.window)
            .font(Some(&data.font))
            .build(&mut data.label_path1)?;

        nwg::TextInput::builder()
            .text("")
            .readonly(false)
            .parent(&data.window)
            .font(Some(&data.font))
            .build(&mut data.text_path1)?;

        nwg::Label::builder()
            .text("文件/文件夹 2:")
            .parent(&data.window)
            .font(Some(&data.font))
            .build(&mut data.label_path2)?;

        nwg::TextInput::builder()
            .text("")
            .readonly(false)
            .parent(&data.window)
            .font(Some(&data.font))
            .build(&mut data.text_path2)?;

        nwg::Button::builder()
            .text("互换名称")
            .parent(&data.window)
            .font(Some(&data.font))
            .build(&mut data.btn_exchange)?;

        let ui = AppUi {
            inner: Rc::new(data),
            default_handler: RefCell::new(None),
        };

        nwg::FlexboxLayout::builder()
            .parent(&ui.inner.window)
            .flex_direction(FlexDirection::Column)
            .padding(PADDING)
            .child(&ui.inner.label_path1)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(20.0),
            })
            .child(&ui.inner.text_path1)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(30.0),
            })
            .child(&ui.inner.label_path2)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(20.0),
            })
            .child(&ui.inner.text_path2)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(30.0),
            })
            .child(&ui.inner.btn_exchange)
            .child_margin(MARGIN)
            .child_flex_grow(1.0)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Auto,
            })
            .build(&ui.inner.layout)?;

        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(evt_ui) = evt_ui.upgrade() {
                match evt {
                    E::OnButtonClick => {
                        if handle == evt_ui.btn_exchange {
                            evt_ui.on_exchange();
                        }
                    }
                    E::OnWindowClose => {
                        if handle == evt_ui.window {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    E::OnInit => {
                        setup_window_style(&evt_ui.window);
                        setup_drag_drop(&evt_ui);
                    }
                    _ => {}
                }
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.inner.window.handle,
            handle_events,
        ));

        Ok(ui)
    }
}

fn setup_window_style(window: &nwg::Window) {
    if let Some(hwnd) = window.handle.hwnd() {
        unsafe {
            let style = winapi::um::winuser::GetWindowLongPtrW(hwnd as _, GWL_STYLE);
            let new_style = style & !(WS_MAXIMIZEBOX as isize);
            winapi::um::winuser::SetWindowLongPtrW(hwnd as _, GWL_STYLE, new_style);

            let ex_style = winapi::um::winuser::GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE);
            // let new_ex_style = ex_style | (WS_EX_TOOLWINDOW as isize) | (WS_EX_TOPMOST as isize);
            // Optionally make it topmost if desired
            let new_ex_style = ex_style | (WS_EX_TOPMOST as isize);
            winapi::um::winuser::SetWindowLongPtrW(hwnd as _, GWL_EXSTYLE, new_ex_style);

            let screen_width =
                winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CXSCREEN);
            let screen_height =
                winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CYSCREEN);

            let mut rect: winapi::shared::windef::RECT = std::mem::zeroed();
            winapi::um::winuser::GetWindowRect(hwnd as _, &mut rect);
            let window_width = rect.right - rect.left;
            let window_height = rect.bottom - rect.top;

            let x = (screen_width - window_width) / 2;
            let y = (screen_height - window_height) / 2;

            winapi::um::winuser::SetWindowPos(
                hwnd as _,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                winapi::um::winuser::SWP_NOSIZE | winapi::um::winuser::SWP_SHOWWINDOW,
            );
        }
    }
}

fn setup_drag_drop(app: &App) {
    if let Some(hwnd) = app.window.handle.hwnd() {
        unsafe {
            winapi::um::shellapi::DragAcceptFiles(hwnd as _, 1);

            let old_proc = winapi::um::winuser::SetWindowLongPtrW(
                hwnd as _,
                winapi::um::winuser::GWLP_WNDPROC,
                subclass_wnd_proc as *const () as isize,
            );
            OLD_WND_PROC = Some(old_proc);

            let app_ptr = app as *const App as isize;
            winapi::um::winuser::SetWindowLongPtrW(
                hwnd as _,
                winapi::um::winuser::GWLP_USERDATA,
                app_ptr,
            );
        }
    }
}

unsafe extern "system" fn subclass_wnd_proc(
    hwnd: winapi::shared::windef::HWND,
    msg: u32,
    wparam: winapi::shared::minwindef::WPARAM,
    lparam: winapi::shared::minwindef::LPARAM,
) -> isize {
    if msg == WM_GETMINMAXINFO {
        let minmax = lparam as *mut winapi::um::winuser::MINMAXINFO;
        (*minmax).ptMinTrackSize.x = MIN_WIDTH;
        (*minmax).ptMinTrackSize.y = MIN_HEIGHT;
        return 0;
    }

    if msg == WM_DROPFILES {
        let hdrop = wparam as winapi::um::shellapi::HDROP;
        let count =
            winapi::um::shellapi::DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);

        let mut paths = Vec::new();
        for i in 0..count {
            let mut buffer = [0u16; MAX_PATH + 1];
            winapi::um::shellapi::DragQueryFileW(hdrop, i, buffer.as_mut_ptr(), MAX_PATH as u32);
            let path_str = String::from_utf16_lossy(
                &buffer[..buffer.iter().position(|&c| c == 0).unwrap_or(MAX_PATH)],
            );
            paths.push(PathBuf::from(path_str));
        }

        let app_ptr =
            winapi::um::winuser::GetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA);
        if app_ptr != 0 {
            let app_ref = &*(app_ptr as *const App);
            app_ref.drop_files(paths);
        }

        winapi::um::shellapi::DragFinish(hdrop);
        return 0;
    }

    if let Some(old_proc) = OLD_WND_PROC {
        winapi::um::winuser::CallWindowProcW(
            mem::transmute::<
                isize,
                std::option::Option<
                    unsafe extern "system" fn(
                        *mut winapi::shared::windef::HWND__,
                        u32,
                        usize,
                        isize,
                    ) -> isize,
                >,
            >(old_proc),
            hwnd as _,
            msg,
            wparam as _,
            lparam as _,
        )
    } else {
        winapi::um::winuser::DefWindowProcW(hwnd as _, msg, wparam as _, lparam as _)
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let default_font = nwg::Font::default();
    nwg::Font::set_global_family("Microsoft YaHei UI").unwrap();
    nwg::Font::set_global_default(Some(default_font));

    let _app = App::build_ui(App {
        window: Default::default(),
        font: Default::default(),
        label_path1: Default::default(),
        text_path1: Default::default(),
        label_path2: Default::default(),
        text_path2: Default::default(),
        btn_exchange: Default::default(),
        layout: Default::default(),
        path1: RefCell::new(None),
        path2: RefCell::new(None),
    })
    .expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
