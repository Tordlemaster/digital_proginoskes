use setup::setup_main;
use render::render_main;

mod setup;
mod render;


fn main() {
    setup_main(false, false, false);
    render_main();
}
