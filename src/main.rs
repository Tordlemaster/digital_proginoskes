use setup::setup_main;

mod setup;
mod render;

const SCR_WIDTH: u32 = 1920;
const SCR_HEIGHT: u32 = 1080;

fn main() {
    setup_main(false, false);
    //println!("Hello, world!");
}
