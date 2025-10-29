use std::cell::RefCell;

use slint::{ComponentHandle, SharedString};

slint::include_modules!();

std::thread_local! {
    static ACTIVE_MSGBOX: RefCell<Option<MsgBoxWindow>> = RefCell::new(None);
}

pub fn info_msgbox(message: &str, caption: &str, _flags: u32) {
    let message_ss = SharedString::from(message);
    let caption_ss = SharedString::from(caption);

    let invoke_result = slint::invoke_from_event_loop({
        let message = message_ss.clone();
        let caption = caption_ss.clone();
        move || {
            show_msgbox(message, caption);
        }
    });

    if invoke_result.is_err() {
        show_msgbox_and_wait(message_ss, caption_ss);
    }
}

fn show_msgbox(message: SharedString, caption: SharedString) {
    ACTIVE_MSGBOX.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(existing) = slot.as_ref() {
            existing.set_message(message.clone());
            existing.set_caption(caption.clone());
            if let Err(err) = existing.show() {
                eprintln!("Failed to show MsgBoxWindow: {err}");
            }
        } else {
            let handle = MsgBoxWindow::new().expect("Failed to create MsgBoxWindow");
            handle.set_message(message);
            handle.set_caption(caption);
            if let Err(err) = handle.show() {
                eprintln!("Failed to show MsgBoxWindow: {err}");
            }
            *slot = Some(handle);
        }
    });
}

fn show_msgbox_and_wait(message: SharedString, caption: SharedString) {
    let handle = MsgBoxWindow::new().expect("Failed to create MsgBoxWindow");
    handle.set_message(message);
    handle.set_caption(caption);
    if let Err(err) = handle.run() {
        eprintln!("Failed to run MsgBoxWindow: {err}");
    }
}
