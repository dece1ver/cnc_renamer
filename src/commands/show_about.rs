use cnc_renamer::return_back;

pub fn show_about() {
    clearscreen::clear().unwrap();
    print!("{}", include_str!("../../res/about"));
    return_back()
}
