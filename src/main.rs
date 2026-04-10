#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::mem;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;

mod msgbox {
    include!("../lib/msgbox.rs");
}

use name_exchanger_lib::exchange_rs;
use native_windows_gui as nwg;
use nwg::NativeUi;

const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_APPWINDOW: u32 = 0x00040000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WM_DROPFILES: u32 = 0x0233;
const WM_GETMINMAXINFO: u32 = 0x0024;
const WM_NCHITTEST: u32 = 0x0084;
const HTCAPTION: isize = 2;
const MAX_PATH: usize = 260;
const MIN_WIDTH: i32 = 450;
const MIN_HEIGHT: i32 = 250;
const TITLE_BAR_HEIGHT: i32 = 44;
const HWND_TOPMOST: isize = -1;
const HWND_NOTOPMOST: isize = -2;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_NOOWNERZORDER: u32 = 0x0200;
const SW_MINIMIZE: i32 = 6;
const SW_RESTORE: i32 = 9;
const THEME_COLOR: [u8; 3] = [0x00, 0x78, 0xD7];

static mut OLD_WND_PROC: Option<isize> = None;

pub struct App {
    window: nwg::Window,
    font: nwg::Font,
    small_font: nwg::Font,

    title_bar: nwg::Frame,
    title_label: nwg::Label,
    btn_pin: nwg::Button,
    btn_help: nwg::Button,
    btn_minimize: nwg::Button,
    btn_close: nwg::Button,
    title_layout: nwg::FlexboxLayout,

    content: nwg::Frame,
    label_path1: nwg::Label,
    text_path1: nwg::TextInput,

    label_path2: nwg::Label,
    text_path2: nwg::TextInput,

    btn_exchange: nwg::Button,
    content_layout: nwg::FlexboxLayout,
    main_layout: nwg::FlexboxLayout,

    tray_icon: nwg::Icon,
    tray: nwg::TrayNotification,

    path1: RefCell<Option<PathBuf>>,
    path2: RefCell<Option<PathBuf>>,
    is_topmost: RefCell<bool>,
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

    fn refresh_pin_button(&self) {
        let text = if *self.is_topmost.borrow() {
            "取消置顶"
        } else {
            "置顶"
        };
        self.btn_pin.set_text(text);
    }

    fn apply_topmost(&self, enabled: bool) {
        if let Some(hwnd) = self.window.handle.hwnd() {
            unsafe {
                winapi::um::winuser::SetWindowPos(
                    hwnd as _,
                    if enabled {
                        HWND_TOPMOST as _
                    } else {
                        HWND_NOTOPMOST as _
                    },
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
                );
            }
        }
    }

    fn toggle_topmost(&self) {
        let new_state = !*self.is_topmost.borrow();
        *self.is_topmost.borrow_mut() = new_state;
        self.apply_topmost(new_state);
        self.refresh_pin_button();
    }

    fn show_help(&self) {
        msgbox::info_msgbox(
            "拖入文件或文件夹即可使用；点击“置顶”切换置顶状态；点击“-”最小化到任务栏，点击“X”隐藏到托盘；左键单击托盘图标可显示/隐藏窗口，右键托盘图标退出。",
            "使用提示",
            0,
        );
    }

    fn minimize_window(&self) {
        if let Some(hwnd) = self.window.handle.hwnd() {
            unsafe {
                winapi::um::winuser::ShowWindow(hwnd as _, SW_MINIMIZE);
            }
        }
    }

    fn hide_window(&self) {
        self.window.set_visible(false);
    }

    fn show_window(&self) {
        self.window.set_visible(true);
        if let Some(hwnd) = self.window.handle.hwnd() {
            unsafe {
                winapi::um::winuser::ShowWindow(hwnd as _, SW_RESTORE);
            }
        }
        self.window.set_focus();
    }

    fn toggle_visibility(&self) {
        if self.window.visible() {
            self.hide_window();
        } else {
            self.show_window();
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.tray.handle.destroy();
        self.window.handle.destroy();
    }
}

impl nwg::NativeUi<AppUi> for App {
    fn build_ui(mut data: App) -> Result<AppUi, nwg::NwgError> {
        use nwg::stretch::{
            geometry::{Rect, Size},
            style::{AlignItems, Dimension as D, FlexDirection, JustifyContent},
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

        const TITLE_PADDING: Rect<D> = Rect {
            start: D::Points(10.0),
            end: D::Points(10.0),
            top: D::Points(8.0),
            bottom: D::Points(8.0),
        };

        nwg::Window::builder()
            .size((MIN_WIDTH, MIN_HEIGHT))
            .position((0, 0))
            .title("名称交换器")
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut data.window)?;

        nwg::Font::builder()
            .family("Microsoft YaHei UI")
            .size(20)
            .weight(400)
            .build(&mut data.font)?;

        nwg::Font::builder()
            .family("Microsoft YaHei UI")
            .size(14)
            .weight(400)
            .build(&mut data.small_font)?;

        nwg::Frame::builder()
            .background_color(Some(THEME_COLOR))
            .parent(&data.window)
            .build(&mut data.title_bar)?;

        nwg::Label::builder()
            .text("名称交换器")
            .font(Some(&data.font))
            .text_color(Some([255, 255, 255]))
            .parent(&data.title_bar)
            .build(&mut data.title_label)?;

        nwg::Button::builder()
            .text("置顶")
            .background_color(Some(THEME_COLOR))
            .text_color(Some([255, 255, 255]))
            .parent(&data.title_bar)
            .font(Some(&data.small_font))
            .build(&mut data.btn_pin)?;

        nwg::Button::builder()
            .text("帮助")
            .background_color(Some(THEME_COLOR))
            .text_color(Some([255, 255, 255]))
            .parent(&data.title_bar)
            .font(Some(&data.small_font))
            .build(&mut data.btn_help)?;

        nwg::Button::builder()
            .text("-")
            .background_color(Some(THEME_COLOR))
            .text_color(Some([255, 255, 255]))
            .parent(&data.title_bar)
            .font(Some(&data.small_font))
            .build(&mut data.btn_minimize)?;

        nwg::Button::builder()
            .text("X")
            .background_color(Some(THEME_COLOR))
            .text_color(Some([255, 255, 255]))
            .parent(&data.title_bar)
            .font(Some(&data.small_font))
            .build(&mut data.btn_close)?;

        nwg::Frame::builder()
            .background_color(Some([245, 245, 245]))
            .parent(&data.window)
            .build(&mut data.content)?;

        nwg::Label::builder()
            .text("文件/文件夹 1:")
            .parent(&data.content)
            .font(Some(&data.font))
            .build(&mut data.label_path1)?;

        nwg::TextInput::builder()
            .text("")
            .readonly(false)
            .parent(&data.content)
            .font(Some(&data.font))
            .build(&mut data.text_path1)?;

        nwg::Label::builder()
            .text("文件/文件夹 2:")
            .parent(&data.content)
            .font(Some(&data.font))
            .build(&mut data.label_path2)?;

        nwg::TextInput::builder()
            .text("")
            .readonly(false)
            .parent(&data.content)
            .font(Some(&data.font))
            .build(&mut data.text_path2)?;

        nwg::Button::builder()
            .text("互换名称")
            .parent(&data.content)
            .font(Some(&data.font))
            .build(&mut data.btn_exchange)?;

        nwg::Icon::builder()
            .source_bin(Some(include_bytes!("../res.ico")))
            .build(&mut data.tray_icon)?;

        nwg::TrayNotification::builder()
            .parent(&data.window)
            .icon(Some(&data.tray_icon))
            .tip(Some("名称交换器"))
            .build(&mut data.tray)?;

        let ui = AppUi {
            inner: Rc::new(data),
            default_handler: RefCell::new(None),
        };

        nwg::FlexboxLayout::builder()
            .parent(&ui.inner.title_bar)
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::FlexEnd)
            .align_items(AlignItems::Center)
            .padding(TITLE_PADDING)
            .child(&ui.inner.title_label)
            .child_flex_grow(1.0)
            .child_size(Size {
                width: D::Auto,
                height: D::Auto,
            })
            .child(&ui.inner.btn_pin)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Points(70.0),
                height: D::Percent(1.0),
            })
            .child(&ui.inner.btn_help)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Points(70.0),
                height: D::Percent(1.0),
            })
            .child(&ui.inner.btn_minimize)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Points(50.0),
                height: D::Percent(1.0),
            })
            .child(&ui.inner.btn_close)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Points(50.0),
                height: D::Percent(1.0),
            })
            .build(&ui.inner.title_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&ui.inner.content)
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
            .build(&ui.inner.content_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&ui.inner.window)
            .flex_direction(FlexDirection::Column)
            .child(&ui.inner.title_bar)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Points(TITLE_BAR_HEIGHT as f32),
            })
            .child(&ui.inner.content)
            .child_flex_grow(1.0)
            .child_size(Size {
                width: D::Percent(1.0),
                height: D::Auto,
            })
            .build(&ui.inner.main_layout)?;

        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(evt_ui) = evt_ui.upgrade() {
                match evt {
                    E::OnButtonClick => {
                        if handle == evt_ui.btn_exchange {
                            evt_ui.on_exchange();
                        } else if handle == evt_ui.btn_minimize {
                            evt_ui.minimize_window();
                        } else if handle == evt_ui.btn_close {
                            evt_ui.hide_window();
                        } else if handle == evt_ui.btn_pin {
                            evt_ui.toggle_topmost();
                        } else if handle == evt_ui.btn_help {
                            evt_ui.show_help();
                        }
                    }
                    E::OnMousePress(nwg::MousePressEvent::MousePressLeftUp) => {
                        if handle == evt_ui.tray {
                            evt_ui.toggle_visibility();
                        }
                    }
                    E::OnContextMenu => {
                        if handle == evt_ui.tray {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    E::OnWindowClose => {
                        if handle == evt_ui.window {
                            evt_ui.hide_window();
                        }
                    }
                    E::OnInit => {
                        setup_window_style(&evt_ui.window, *evt_ui.is_topmost.borrow());
                        setup_drag_drop(&evt_ui);
                        evt_ui.refresh_pin_button();
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

fn setup_window_style(window: &nwg::Window, topmost: bool) {
    if let Some(hwnd) = window.handle.hwnd() {
        unsafe {
            let style = winapi::um::winuser::GetWindowLongPtrW(hwnd as _, GWL_STYLE) as u32;
            let mut new_style = style;
            new_style &= !(WS_MAXIMIZEBOX | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU);
            new_style |= WS_MINIMIZEBOX;
            winapi::um::winuser::SetWindowLongPtrW(hwnd as _, GWL_STYLE, new_style as isize);

            let ex_style = winapi::um::winuser::GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE) as u32;
            let mut new_ex_style = ex_style | WS_EX_APPWINDOW;
            if topmost {
                new_ex_style |= WS_EX_TOPMOST;
            } else {
                new_ex_style &= !WS_EX_TOPMOST;
            }
            winapi::um::winuser::SetWindowLongPtrW(hwnd as _, GWL_EXSTYLE, new_ex_style as isize);

            winapi::um::winuser::SetWindowPos(
                hwnd as _,
                if topmost {
                    HWND_TOPMOST as _
                } else {
                    HWND_NOTOPMOST as _
                },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
            );

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
                ptr::null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOACTIVATE,
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
    if msg == WM_NCHITTEST {
        let x = (lparam & 0xFFFF) as i16 as i32;
        let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
        let mut point = winapi::shared::windef::POINT { x, y };
        unsafe {
            winapi::um::winuser::ScreenToClient(hwnd, &mut point);
        }
        if point.y >= 0 && point.y < TITLE_BAR_HEIGHT {
            return HTCAPTION;
        }
    }

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
        small_font: Default::default(),
        title_bar: Default::default(),
        title_label: Default::default(),
        btn_pin: Default::default(),
        btn_help: Default::default(),
        btn_minimize: Default::default(),
        btn_close: Default::default(),
        title_layout: Default::default(),
        content: Default::default(),
        label_path1: Default::default(),
        text_path1: Default::default(),
        label_path2: Default::default(),
        text_path2: Default::default(),
        btn_exchange: Default::default(),
        content_layout: Default::default(),
        main_layout: Default::default(),
        tray_icon: Default::default(),
        tray: Default::default(),
        path1: RefCell::new(None),
        path2: RefCell::new(None),
        is_topmost: RefCell::new(true),
    })
    .expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
