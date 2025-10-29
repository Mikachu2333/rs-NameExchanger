slint::include_modules!();

use libs::{init, switch_admin};

fn main() -> Result<(), slint::PlatformError> {
    let (is_admin, mut is_upper) = init();

    let main_window = MainWindow::new()?;
    main_window.set_is_admin(is_admin);

    main_window.on_request_help({
        let main_window_weak = main_window.as_weak();
        move || {
            if let Some(window) = main_window_weak.upgrade() {
                window.set_msgbox_message(
                    libs::show_help().into(),
                );
                //
            }
        }
    });

    main_window.on_admin_switch(move || {
        switch_admin(is_upper);
        is_upper = !is_upper;
    });

    main_window.run()
}
