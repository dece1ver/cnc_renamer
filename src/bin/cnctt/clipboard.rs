use std::io;
use std::ptr;

use windows_sys::Win32::Foundation::{GlobalFree, HANDLE};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};

/// Формат буфера обмена CF_UNICODETEXT.
const CF_UNICODETEXT: u32 = 13;

/// Помещает текст в буфер обмена (CF_UNICODETEXT).
pub fn set_text(text: &str) -> io::Result<()> {
    // SAFETY: после успешного OpenClipboard владение буфером обмена принадлежит
    // текущему потоку до CloseClipboard; `hmem` выделяется через GlobalAlloc и при
    // успешном SetClipboardData передаётся системе (не освобождается здесь);
    // в случае ошибки освобождаем вручную. `utf16` живёт до копирования.
    unsafe {
        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = (|| {
            if EmptyClipboard() == 0 {
                return Err(io::Error::last_os_error());
            }
            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes = utf16.len() * std::mem::size_of::<u16>();
            let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if hmem.is_null() {
                return Err(io::Error::last_os_error());
            }
            let locked = GlobalLock(hmem);
            if locked.is_null() {
                GlobalFree(hmem);
                return Err(io::Error::last_os_error());
            }
            ptr::copy_nonoverlapping(utf16.as_ptr().cast::<u8>(), locked.cast::<u8>(), bytes);
            GlobalUnlock(hmem);
            if SetClipboardData(CF_UNICODETEXT, hmem as HANDLE).is_null() {
                GlobalFree(hmem);
                return Err(io::Error::last_os_error());
            }
            Ok(())
        })();
        CloseClipboard();
        result
    }
}
