slint::include_modules!();

use libs::*;

fn main() -> Result<(), slint::PlatformError> {
    let (is_admin, mut is_upper) = libs::init();

    let main_window = MainWindow::new()?;
    main_window.set_is_admin(is_admin);

    main_window.on_request_help(|| {
        info_msgbox(
            "NameExchanger keeps your chosen window pinned on top.",
            "NameExchanger Help",
            0,
        );
    });

    main_window.on_admin_switch(move || {
        switch_admin(is_upper);
        is_upper = !is_upper;
    });

    main_window.run()
}
