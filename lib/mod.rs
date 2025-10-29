use std::path::Path;

mod exchange;
mod file_rename;
mod msgbox;
mod path_checkout;
mod types;

use crate::exchange::exchange_paths;
#[allow(unused_imports)]
use crate::types::RenameError;
pub use msgbox::info_msgbox;
/// bool: is admin
/// bool: is EXE
pub fn init() -> (bool, bool) {
    let current_exe_path = std::env::current_exe().unwrap();
    let ext: String = current_exe_path
        .extension()
        .unwrap()
        .to_string_lossy()
        .into();
    let upper = match ext.as_str() {
        "exe" => false,
        "EXE" => {
            std::process::exit(0);
        }
        _ => {
            show_help();
            switch_admin(true);
            false
        }
    };
    (is_admin(), upper)
}

pub fn switch_admin(is_upper: bool) -> String {
    let current_exe_path = std::env::current_exe().unwrap();
    let new_ext = if is_upper { ".exe" } else { ".EXE" };
    let exe_name = current_exe_path.file_stem().unwrap().to_string_lossy();
    let new_name = format!("{}{}", exe_name, new_ext);
    let current_str = current_exe_path.to_string_lossy().to_string();
    let new_path = current_str.replace(
        current_exe_path.file_name().unwrap().to_str().unwrap(),
        new_name.as_str(),
    );

    match std::fs::rename(current_str, new_path) {
        Ok(_) => "".to_string(),
        Err(e) => e.to_string(),
    }
}

pub fn show_help() -> String {
    r#"拖入文件即可使用，软件将常驻任务栏，悬停鼠标于
按钮上可获得提示。

软件包含以下功能
〇管理员身份启动
〇创建/删除「发送到」菜单快捷方式
〇置顶

拖入檔案即可使用，軟體將常駐系統匣，懸停鼠標於
按鈕上可獲得提示。

軟體包含以下功能
〇總是以系統管理員執行
〇建立/刪除「傳送到」選單捷徑方式
〇置頂"#
        .to_string()
}

#[cfg(windows)]
#[link(name = "shell32")]
unsafe extern "system" {
    fn IsUserAnAdmin() -> bool;
}

#[cfg(windows)]
pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() }
}

/// Rust interface function for swapping names of two files or directories
///
/// ### Parameters
/// * `path1` - First file or directory path
/// * `path2` - Second file or directory path
///
/// ### Return Value
/// * `Ok(())` - Success
/// * `Err(RenameError)` - Error information
pub fn exchange_rs(path1: &Path, path2: &Path) -> Result<(), types::RenameError> {
    exchange_paths(path1.to_path_buf(), path2.to_path_buf())
}
