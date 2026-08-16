use std::cell::RefCell;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, SendMessageW, WM_GETTEXT,
};

/// Фрагмент заголовка окна CIMCO Edit.
const CIMCO_TITLE: &str = "CIMCO Edit";

thread_local! {
    static HWNDS: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
}

// SAFETY: callback для EnumWindows — вызывается системой синхронно в потоке,
// вызвавшем EnumWindows; HWNDS — thread_local, поэтому перезапись/чтение безопасны.
// Возвращает TRUE, чтобы продолжить перечисление.
unsafe extern "system" fn enum_proc(hwnd: HWND, _lparam: isize) -> i32 {
    HWNDS.with(|h| h.borrow_mut().push(hwnd));
    1
}

/// Возвращает заголовки (WM_GETTEXT) окон верхнего уровня,
/// у которых GetWindowText содержит «CIMCO Edit».
pub fn find_cimco_windows() -> Vec<String> {
    // SAFETY: EnumWindows вызывает enum_proc на текущем потоке; lparam не используется.
    unsafe {
        HWNDS.with(|h| h.borrow_mut().clear());
        EnumWindows(Some(enum_proc), 0);
    }
    HWNDS.with(|h| {
        h.borrow()
            .iter()
            .filter_map(|&hwnd| {
                if get_window_text(hwnd).contains(CIMCO_TITLE) {
                    Some(get_window_text2(hwnd))
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Разбирает заголовок окна CIMCO Edit вида `[C:\...\file.nc]` (или `[C:\...\file.nc*]`,
/// если файл изменён, но не сохранён) на путь и признак наличия незаписанных правок.
pub fn file_path_from_title(title: &str) -> (String, bool) {
    let modified = title.trim_end_matches(']').ends_with('*');
    let path = title
        .split_once('[')
        .map(|(_, rest)| rest)
        .unwrap_or(title)
        .trim_end_matches(']')
        .trim_end_matches('*')
        .trim();
    (path.to_string(), modified)
}

fn get_window_text(hwnd: HWND) -> String {
    // SAFETY: буфер размером len+1; GetWindowTextW пишет завершённый
    // null-терминатором текст и не выходит за пределы буфера.
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
        utf16_to_string(&buf)
    }
}

fn get_window_text2(hwnd: HWND) -> String {
    // SAFETY: WM_GETTEXT ожидает буфер размера (число символов + 1) и пишет
    // завершённую null-терминатором строку; buf имеет нужный размер.
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        SendMessageW(
            hwnd,
            WM_GETTEXT,
            (len + 1) as usize,
            buf.as_mut_ptr() as isize,
        );
        utf16_to_string(&buf)
    }
}

fn utf16_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_saved_title() {
        let (path, modified) = file_path_from_title(r"[C:\NC\part.nc]");
        assert_eq!(path, r"C:\NC\part.nc");
        assert!(!modified);
    }

    #[test]
    fn parses_modified_title() {
        let (path, modified) = file_path_from_title(r"[C:\NC\part.nc*]");
        assert_eq!(path, r"C:\NC\part.nc");
        assert!(modified);
    }

    #[test]
    fn parses_title_without_brackets() {
        let (path, modified) = file_path_from_title(r"C:\NC\part.nc");
        assert_eq!(path, r"C:\NC\part.nc");
        assert!(!modified);
    }
}
