use fltk::{
    app,
    button::Button,
    enums::{Align, Color, Event, Font, FrameType},
    frame::Frame,
    input::Input,
    prelude::*,
    window::Window,
};
use tray_icon::{Icon, MouseButton, TrayIconBuilder, TrayIconEvent};

static DEBUG: bool = cfg!(debug_assertions);

fn main() {
    let app = app::App::default();

    // Load custom font
    let custom_font = Font::load_font("NameExchangeCustom.ttf").unwrap();
    Font::set_font(Font::Screen, &custom_font);

    // Set default font for other elements
    #[cfg(target_os = "windows")]
    let default_font = "Microsoft YaHei UI";
    #[cfg(target_os = "linux")]
    let default_font = "Noto Sans CJK SC";
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    let default_font = "sans-serif";

    let default_font = Font::by_name(default_font);
    app::set_font(default_font);
    app::set_font_size(14);

    let mut win = Window::new(100, 100, 362, 242, "FilenameExchanger");
    win.set_color(Color::from_rgb(240, 240, 240));
    win.set_border(false); // borderless

    // Top bar background
    let mut top_bg = Frame::new(0, 0, 362, 33, "");
    top_bg.set_frame(FrameType::FlatBox);
    let theme_color = Color::from_rgb(0x00, 0x78, 0xD7);
    top_bg.set_color(theme_color);

    // Top bar buttons
    let mut btn_top = Button::new(4, 4, 27, 27, "B");
    btn_top.set_frame(FrameType::FlatBox);
    btn_top.set_color(theme_color);
    btn_top.set_label_color(Color::Yellow);
    btn_top.set_label_font(Font::Screen);
    btn_top.set_label_size(21);
    btn_top.clear_visible_focus();

    let mut btn_tip = Button::new(184, 4, 28, 27, "C");
    btn_tip.set_frame(FrameType::FlatBox);
    btn_tip.set_color(theme_color);
    btn_tip.set_label_color(Color::Red);
    btn_tip.set_label_font(Font::Screen);
    btn_tip.set_label_size(21);
    btn_tip.clear_visible_focus();

    let mut btn_admin = Button::new(222, 4, 27, 27, "D");
    btn_admin.set_frame(FrameType::FlatBox);
    btn_admin.set_color(theme_color);
    btn_admin.set_label_color(Color::White);
    btn_admin.set_label_font(Font::Screen);
    btn_admin.set_label_size(21);
    btn_admin.clear_visible_focus();

    let mut btn_menu = Button::new(258, 4, 27, 27, "F");
    btn_menu.set_frame(FrameType::FlatBox);
    btn_menu.set_color(theme_color);
    btn_menu.set_label_color(Color::White);
    btn_menu.set_label_font(Font::Screen);
    btn_menu.set_label_size(21);
    btn_menu.clear_visible_focus();

    let mut btn_min = Button::new(292, 4, 27, 27, "G");
    btn_min.set_frame(FrameType::FlatBox);
    btn_min.set_color(theme_color);
    btn_min.set_label_color(Color::White);
    btn_min.set_label_font(Font::Screen);
    btn_min.set_label_size(21);
    btn_min.clear_visible_focus();

    let mut btn_close = Button::new(328, 4, 28, 27, "H");
    btn_close.set_frame(FrameType::FlatBox);
    btn_close.set_color(theme_color);
    btn_close.set_label_color(Color::White);
    btn_close.set_label_font(Font::Screen);
    btn_close.set_label_size(21);
    btn_close.clear_visible_focus();

    // Labels
    let mut lbl_path1 = Frame::new(11, 39, 46, 18, "File 1");
    lbl_path1.set_align(Align::Left | Align::Inside);
    lbl_path1.set_label_font(default_font);

    let mut input_path1 = Input::new(11, 59, 340, 40, "");
    input_path1.set_text_font(default_font);
    input_path1.set_text_size(15);

    let mut lbl_path2 = Frame::new(11, 106, 46, 18, "File 2");
    lbl_path2.set_align(Align::Left | Align::Inside);
    lbl_path2.set_label_font(default_font);

    let mut input_path2 = Input::new(11, 126, 340, 40, "");
    input_path2.set_text_font(default_font);
    input_path2.set_text_size(15);

    // Start button
    let mut btn_start = Button::new(118, 183, 127, 44, "Start");
    btn_start.set_label_size(20);
    btn_start.set_label_font(default_font);
    btn_start.clear_visible_focus();

    win.end();
    win.make_resizable(false);
    win = win.center_screen();
    win.show();
    win.set_on_top();

    // Setup tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("FilenameExchanger")
        .with_icon(Icon::from_rgba(include_bytes!("../raw_icon_data").into(), 256, 256).unwrap())
        .build()
        .unwrap();

    // Make window draggable by top bar
    let mut win_clone = win.clone();
    let mut x_offset = 0;
    let mut y_offset = 0;
    top_bg.handle(move |_, ev| match ev {
        Event::Push => {
            let coords = app::event_coords();
            x_offset = coords.0;
            y_offset = coords.1;
            true
        }
        Event::Drag => {
            let x = app::event_x_root();
            let y = app::event_y_root();
            win_clone.set_pos(x - x_offset, y - y_offset);
            true
        }
        _ => false,
    });

    let mut win_clone2 = win.clone();
    btn_close.set_callback(move |_| {
        win_clone2.hide();
    });

    let mut win_clone3 = win.clone();
    btn_min.set_callback(move |_| {
        win_clone3.iconize();
    });

    // Drag and drop support
    win.handle({
        let mut input_path1 = input_path1.clone();
        let mut input_path2 = input_path2.clone();
        move |_, ev| match ev {
            Event::DndEnter | Event::DndDrag | Event::DndRelease => true,
            Event::Paste => {
                let text = app::event_text();
                let files: Vec<&str> = text
                    .lines()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if files.len() == 1 {
                    let file = files[0].trim_matches('"');
                    if input_path1.value().is_empty() {
                        input_path1.set_value(file);
                    } else if input_path2.value().is_empty() {
                        input_path2.set_value(file);
                    } else {
                        input_path1.set_value(file);
                        input_path2.set_value("");
                    }
                } else if files.len() >= 2 {
                    input_path1.set_value(files[0].trim_matches('"'));
                    input_path2.set_value(files[1].trim_matches('"'));
                }
                true
            }
            _ => false,
        }
    });

    let mut win_clone4 = win.clone();
    app::add_idle3(move |_| {
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } => {
                    if win_clone4.shown() {
                        win_clone4.hide();
                        if DEBUG {
                            println!("Window hidden via tray icon.");
                        }
                    } else {
                        win_clone4.show();
                        if DEBUG {
                            println!("Window shown via tray icon.");
                        }
                    }
                }
                TrayIconEvent::Click {
                    button: MouseButton::Right,
                    ..
                } => {
                    if DEBUG {
                        println!("Right-clicked tray icon, exiting.");
                    }
                    app::quit();
                }
                _ => {
                }
            }
        }
    });

    app.run().unwrap();
}
