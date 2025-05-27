use setup::setup_main;
use render::render_main;

mod setup;
mod render;
mod spherical_quadtree;


fn main() {
    let mut quadtree = spherical_quadtree::SphQtRoot::new();
    setup_main(false, false, false, &mut quadtree);
    render_main(&quadtree);
}
