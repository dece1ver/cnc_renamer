#[cfg(winfows)]
extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon_with_id("icons/rename256.ico", "1");
        res.set_icon_with_id("icons/rename48.ico", "2");
        res.set_icon_with_id("icons/rename32.ico", "3");
        res.compile().unwrap();
    }
}
