#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::ffi::c_void;
use eframe::egui::{self, ViewportCommand, WindowLevel};
use exchange_lib::exchange;
use mslnk::ShellLink;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::{ffi::CString, sync::Mutex};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::UI::Shell::{FOLDERID_SendTo, SHGetKnownFolderPath, KF_FLAG_DEFAULT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, ShowWindow, MB_ICONINFORMATION, MB_OK, SW_HIDE, SW_SHOWDEFAULT,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle, Win32WindowHandle};

static VISIBLE: Mutex<bool> = Mutex::new(true);
static WINDOW_HANDLE: AtomicI32 = AtomicI32::new(0);

fn main() -> eframe::Result {
    let args = std::env::args().collect::<Vec<String>>();
    println!("{:?}", args);
    if args.len() == 3 {
        let path1 = CString::new(args[1].clone()).unwrap();
        let path2 = CString::new(args[2].clone()).unwrap();
        let result = exchange(path1.as_ptr(), path2.as_ptr());
        if result == 0 {
            Ok(())
        } else {
            panic!("{}", output_trans(result))
        }
    } else {
        let icon_data = include_bytes!("../raw_icon_data").to_vec();
        let _tray_icon = TrayIconBuilder::new()
            .with_icon(Icon::from_rgba(icon_data, 256, 256).unwrap())
            .with_tooltip("左键显示隐藏，右键退出\n左鍵顯示隱藏，右鍵退出")
            .build()
            .unwrap();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([350.0, 250.0])
                .with_resizable(false)
                .with_taskbar(false)
                .with_decorations(false)
                .with_transparent(true)
                .with_drag_and_drop(true)
                .with_always_on_top(),
            centered: true,
            ..Default::default()
        };

        eframe::run_native(
            "FileNameExchanger",
            options,
            Box::new(|cc| {
                let RawWindowHandle::Win32(handle) = cc.window_handle().unwrap().as_raw() else {
                    panic!("Unsupported platform");
                };

                WINDOW_HANDLE.store(handle_to_hwnd(handle) as i32, Ordering::SeqCst);

                TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| match event {
                    TrayIconEvent::Click {
                        button_state: MouseButtonState::Down,
                        button: MouseButton::Left,
                        ..
                    } => {
                        let mut visible = VISIBLE.lock().unwrap();
                        let hwnd = WINDOW_HANDLE.load(Ordering::SeqCst) as *mut c_void;

                        if *visible {
                            unsafe {
                                let _ = ShowWindow(hwnd, SW_HIDE);
                            }
                            *visible = false;
                        } else {
                            unsafe {
                                let _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
                            }
                            *visible = true;
                        }
                        println!("{visible}");
                    }
                    TrayIconEvent::Click {
                        button_state: MouseButtonState::Down,
                        button: MouseButton::Right,
                        ..
                    } => {
                        std::process::exit(0);
                    }
                    _ => (),
                }));

                load_fonts(&cc.egui_ctx);
                Ok(Box::<MyApp>::default())
            }),
        )
    }
}

struct MyApp {
    path_string1: String,
    path_string2: String,
    result_string: String,
    on_top: String,
    drop_counter: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            path_string1: String::new(),
            path_string2: String::new(),
            result_string: "输出 輸出".to_owned(),
            on_top: "⚓".to_owned(),
            drop_counter: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            ctx.input(|i| {
                for dropped_file in &i.raw.dropped_files {
                    if let Some(file_path) = &dropped_file.path {
                        let path_string = file_path.to_string_lossy().to_string();
                        if self.drop_counter {
                            self.path_string1 = path_string;
                        } else {
                            self.path_string2 = path_string;
                        }
                        self.drop_counter = !self.drop_counter;
                    }
                }
            });
        }

        egui::TopBottomPanel::top("title bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let btn_top = ui.button(&self.on_top);
                if btn_top.on_hover_text("置顶开关\n置頂開關").clicked() {
                    if self.on_top == "⚓" {
                        self.on_top = "🔱".to_string();
                        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
                        self.result_string = "已取消置顶 已取消置頂".to_owned();
                    } else {
                        self.on_top = "⚓".to_owned();
                        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                            WindowLevel::AlwaysOnTop,
                        ));
                        self.result_string = "已置顶 已置頂".to_owned();
                    }
                };

                let drag_response =
                    ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
                if drag_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("\u{274E}").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("\u{25BC}").on_hover_text("最小化").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));//很怪，不这样没法重置状态
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        let mut visible = VISIBLE.lock().unwrap();
                        *visible = false;
                    }
                    let btn_fix = ui.button("\u{2699}");//不能直接使用图标，很怪
                    if btn_fix.clicked_by(egui::PointerButton::Primary) {
                        creat_lnk(true);
                        self.result_string = "已重置SendTo".to_owned();
                    }
                    else if btn_fix.clicked_by(egui::PointerButton::Secondary){
                        creat_lnk(false);
                        self.result_string = "删除SendTo".to_owned();
                    };
                    btn_fix.on_hover_text("单击可新增右键菜单-“发送到”选项\n右键点击以删除\n\n點擊新增至右鍵選單-“傳送到”選項\n點擊右鍵取消\0");

                    if ui.button(" ? ").clicked() {
                        let hwnd = WINDOW_HANDLE.load(Ordering::SeqCst);
                        let title = "帮助 幫助\0".encode_utf16().collect::<Vec<u16>>();
                        let content = "拖入文件即可使用，软件将常驻任务栏，悬停鼠标于按钮\n上可获得提示。\n点击⚙后将增加“右键菜单-发送到”，支持直接右键使用。\n如遇右键菜单失效，请右键点击⚙。\n左上角⚓/🔱是置顶开关。\n\n拖入檔案即可使用，軟體將常駐系統匣，懸停鼠標於按鈕\n上可獲得提示。\n點擊⚙後可新增功能“右鍵選單-傳送到”，支援直接右鍵使用。\n如果右鍵選單失效，請右鍵點擊⚙。\n左上方⚓/🔱是置頂開關。\0"
                            .encode_utf16()
                            .collect::<Vec<u16>>();

                        unsafe {
                            MessageBoxW(
                                hwnd as *mut c_void,
                                content.as_ptr(),
                                title.as_ptr(),
                                MB_OK | MB_ICONINFORMATION
                            );
                        }
                    };
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("文件1：");
                    egui::ScrollArea::horizontal()
                        .id_salt("scroll1")
                        .show(ui, |ui| {
                            egui::TextEdit::singleline(&mut self.path_string1)
                                .clip_text(false)
                                .show(ui);
                        });
                });
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label("文件2：");
                    egui::ScrollArea::horizontal()
                        .id_salt("scroll2")
                        .show(ui, |ui| {
                            egui::TextEdit::singleline(&mut self.path_string2)
                                .clip_text(false)
                                .show(ui);
                        });
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                let btn_exchange =
                    ui.add(egui::Button::new("交换 交換").min_size(egui::vec2(80.0, 35.0)));
                if btn_exchange.clicked() {
                    let path1 = CString::new(self.path_string1.clone()).unwrap();
                    let path2 = CString::new(self.path_string2.clone()).unwrap();
                    self.result_string = output_trans(exchange(path1.as_ptr(), path2.as_ptr()));
                    if self.result_string == "Success".to_owned() {
                        self.path_string1.clear();
                        self.path_string2.clear();
                    }
                }
            });
        });
        egui::TopBottomPanel::bottom("state label").show(ctx, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.result_string));
        });
    }
}

// 将错误码转换为友好的提示文本
fn output_trans(num: i32) -> String {
    match num {
        0 => "Success",
        1 => "Path not exist",
        2 => "Permission Denied",
        3 => "New File Already Exists",
        _ => "Unknown Error",
    }
    .to_string()
}

// 加载系统字体
fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "msfont".to_owned(),
        egui::FontData::from_owned(std::fs::read("C:/Windows/Fonts/msyh.ttc").unwrap()).into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "msfont".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("msfont".to_owned());
    ctx.set_fonts(fonts);
    ctx.set_pixels_per_point(1.2);
}

// 将窗口句柄转换为 HWND 类型
fn handle_to_hwnd(handle: Win32WindowHandle) -> windows_sys::Win32::Foundation::HWND {
    let hwnd_isize: isize = handle.hwnd.into();
    let hwnd = hwnd_isize as *mut c_void;

    hwnd as windows_sys::Win32::Foundation::HWND
}

// 获取宽字符串长度
fn wcslen(ptr: *const u16) -> usize {
    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    len
}

// 获取 SendTo 文件夹中快捷方式的路径
fn get_lnk_path() -> PathBuf {
    let mut path_ptr = std::ptr::null_mut();
    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_SendTo,
            KF_FLAG_DEFAULT.try_into().unwrap(),
            std::ptr::null_mut(),
            &mut path_ptr,
        )
    };

    if result == 0 {
        let sendto_dir = unsafe {
            let wide_str = std::slice::from_raw_parts(path_ptr, wcslen(path_ptr));
            let os_str =
                <std::ffi::OsString as std::os::windows::ffi::OsStringExt>::from_wide(wide_str);
            PathBuf::from(os_str)
        };
        unsafe {
            windows_sys::Win32::System::Com::CoTaskMemFree(path_ptr as *mut _);
        };
        sendto_dir.join("name_exchanger.lnk")
    } else {
        panic!("Get Sendto Folder failed")
    }
}

// 创建或删除快捷方式
fn creat_lnk(mode: bool) {
    let path = get_lnk_path();
    let _ = std::fs::remove_file(&path);
    if mode {
        let sl =
            ShellLink::new(std::env::args().collect::<Vec<String>>().first().unwrap()).unwrap();
        sl.create_lnk(&path).unwrap();
    }
}
