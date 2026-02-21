use fltk::{app, enums::Font};

fn main() {
    let app = app::App::default();
    let font = app::load_font("NameExchangeCustom.ttf").unwrap();
    println!("Loaded font: {}", font);
}