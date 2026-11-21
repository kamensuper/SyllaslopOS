#![allow(dead_code)]

mod bootinfo;
mod console;
mod graphics;
mod input;
mod io;
mod serial;

fn halt() -> ! {
    unreachable!("the host tests never halt the machine")
}
