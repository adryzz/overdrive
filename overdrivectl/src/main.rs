use std::ptr::null;

use x11::{xrandr, xlib::{XOpenDisplay, XDefaultScreen, self}};

fn main() {
    println!("Hello, world!");
    let a = cvt_utils::CvtTimings::generate(
        1280,
        1024,
        60.0,
        cvt_utils::BlankingMode::ReducedV2,
        false,
        false,
    );
    let a = a.generate_modeline();
    println!("{}", &a);

unsafe {

    let dpy = xlib::XOpenDisplay(null());
    if dpy.is_null() {
        panic!("aaa");
    }
    let screen = xlib::XDefaultScreen(dpy);
    let root = xlib::XRootWindow(dpy, screen);
    let res = xrandr::XRRGetScreenResourcesCurrent(dpy, root);

    dbg!((*res).modes);
}
}
