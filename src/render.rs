//When doing raytracing:
//Keep track of solid angle for each pixel's ray
//If triangle in quadtree is smaller than solid angle, take the average of all the star colors in the triangle and use that as color
//Does this mean maybe the quadtree should be constructed from scratch every launch with the solid angle of the pixels at the set
//resolution in mind? To use as a depth limit for the tree and remove the requirement for the fragment shader to do averages of an
//unbounded number of stars every frame

//Stars themselves have a solid angle (at least a perceptual one based on naked eye fov, that is invariant(doesn't change rel. to screen) to zoom level)

use std::{cmp::max, num, u64};

use glfw::{fail_on_errors, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use utils::{DEBUG_QT_UNI_DEPTH, DEBUG_QT_UNI_STARS, DEBUG_QT_UNI_TRANS};
use crate::spherical_quadtree::{SphQtNode, SphQtRoot};

mod utils;

const SCR_WIDTH: u32 = 1920;
const SCR_HEIGHT: u32 = 1080;

struct WindowData {
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>
}

fn draw_debug_quadtree_recur(parent: &Box<SphQtNode>, recursive_transformation: &glam::Mat3, max_side_stars: u64, depth: i32) {
    //Transformation matrix should be passed down and applied and modified recursively TODO TODO TODO TODO
    //let edge_len = ((parent.corners[1][0] - parent.corners[0][0]) / 2.0).abs();
    let edge_len = 1.0 / (depth + 1) as f32;
    let new_transformation = glam::Mat3::from_scale(glam::vec2(edge_len, edge_len)) * glam::Mat3::from_translation(glam::vec2((parent.corners[1][0] + 1.0) / 2.0, (parent.corners[1][1] + 1.0) / 2.0));
    //let scale = glam::Mat3::from_scale(glam::vec2(edge_len, edge_len));
    //let translation  = glam::Mat3::from_translation(glam::vec2((parent.corners[1][0] + 1.0) / 2.0, (parent.corners[1][1] + 1.0) / 2.0));
    //let mut transformation = scale * translation * (*recursive_transformation);
    //let mut transformation = translation * (*recursive_transformation);
    let transformation = *recursive_transformation * new_transformation;
    //let self_transformation = transformation * glam::Mat3::from_scale(glam::vec2(2.0, 2.0)) * glam::Mat3::from_translation(glam::vec2(-0.5, -0.5));

    println!("Layer {}, Position {} {}, Scale {}", depth, (parent.corners[1][0] + 1.0) / 2.0, (parent.corners[1][1] + 1.0) / 2.0, edge_len);
    println!("Matrix: {}", transformation);
    println!("Self Matrix: {}", transformation);
    //Draw parent
    let area = edge_len * edge_len;

    let mut stars_not_in_children = parent.stars_in_children;
    let mut child_count = 0;

    if depth < 1 {
        for c in &parent.children {
            if c.is_some() {
                child_count += 1;
                let c_uw = c.as_ref().unwrap();
                stars_not_in_children -= c_uw.stars_in_children;

                //Recur into the child
                //draw_debug_quadtree_recur(c_uw, &transformation, max_side_stars, depth + 1);
            }
        }
    }

    let star_density = (stars_not_in_children / max_side_stars) as f32 / (area - ((area / 2.0) * child_count as f32));
    println!("{} stars", star_density);

    unsafe {
        gl::UseProgram(utils::DEBUG_QT_PROGRAM);
        gl::Uniform1f(DEBUG_QT_UNI_STARS, star_density); //Represents the density of stars in the area
        gl::UniformMatrix3fv(DEBUG_QT_UNI_TRANS, 1, gl::FALSE, transformation.as_ref() as *const f32);
        gl::Uniform1i(DEBUG_QT_UNI_DEPTH, depth);
        gl::BindVertexArray(utils::QUAD_VAO);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        gl::BindVertexArray(0);
    }

    //Draw children in the front and parents in layers behind
    //let mut stars_not_in_children = parent.stars_in_children;
    //let mut child_count = 0;
    if depth < 1 {
        for c in &parent.children {
            if c.is_some() {
                let c_uw = c.as_ref().unwrap();

                //Recur into the child
                draw_debug_quadtree_recur(c_uw, &transformation, max_side_stars, depth + 1);
            }
        }
    }
}

fn draw_debug_quadtree(quadtree: &SphQtRoot) {

    //scale to base size
    //scale to 1 / (2^depth)
    //translate to large square base location
    //translate within large square to final location

    let base_scale = glam::vec2(0.2, 0.2);
    let base_positions = [
        glam::vec2(0.3, 0.4), //N
        glam::vec2(0.7, 0.4), //S
        glam::vec2(0.5, 0.4), //E
        glam::vec2(0.1, 0.4), //W
        glam::vec2(0.3, 0.6), //T
        glam::vec2(0.3, 0.2), //B
    ];

    //Calculate the highest number of stars any side has
    let mut max_side_stars = 0;
    for side in &quadtree.faces {
        max_side_stars = max(max_side_stars, side.as_ref().unwrap().stars_in_children);
    }
    
    for i in 0..1 {
        //Depth first search
        //Go deep until reach Option<None>
        let side = &quadtree.faces[i];

        let mut transformation = glam::Mat3::from_translation(base_positions[i]) * glam::Mat3::from_scale(base_scale);
        transformation = glam::Mat3::from_scale(glam::vec2(SCR_HEIGHT as f32 / SCR_WIDTH as f32, 1.0)) * glam::Mat3::from_scale(glam::vec2(2.0, 2.0)) * glam::Mat3::from_translation(glam::vec2(-0.5, -0.5)) * transformation;
        //transformation *= glam::Mat3::from_translation(base_positions[i]);
        //transformation *= glam::Mat3::from_scale(glam::vec2(SCR_HEIGHT as f32 / SCR_WIDTH as f32, 1.0)); //Screen resolution

        //transformation = glam::Mat3::IDENTITY;

        draw_debug_quadtree_recur(side.as_ref().unwrap(), &transformation, max_side_stars, 0);
    }

    

    unsafe {
        gl::UseProgram(utils::DEBUG_QT_PROGRAM);
        gl::BindVertexArray(utils::QUAD_VAO);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        gl::BindVertexArray(0);
    }
}

fn render_setup(gl_context: &mut Glfw, scr_width: u32, scr_height: u32) -> WindowData {
    gl_context.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    gl_context.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    #[cfg(target_os = "macos")] {
        gl_context.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    }

    let (mut window, events) = gl_context.create_window(scr_width, scr_height, "Digital Progonoskes", glfw::WindowMode::Windowed).expect("Failed to create window");
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|s| window.get_proc_address(s));

    utils::init_utils();

    utils::setup_debug_qt_program();

    WindowData { window: window, events: events }
}

fn render_loop(gl_context: &mut Glfw, wd: &mut WindowData, render_draw: fn(&SphQtRoot), quadtree: &SphQtRoot) {
    render_draw(quadtree);
    wd.window.swap_buffers();
    while !wd.window.should_close() {

        //render_draw(quadtree);

        //Take keyboard input
        gl_context.poll_events();
        for (_, event) in glfw::flush_messages(&wd.events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) =>
                    wd.window.set_should_close(true),
                _ => {}
            }
        }

        //Swap buffers
        //wd.window.swap_buffers();
    }
}

pub fn render_main(quadtree: &SphQtRoot) {
    let mut gl_context = glfw::init(fail_on_errors!()).unwrap();
    let mut window_data = render_setup(&mut gl_context, SCR_WIDTH, SCR_HEIGHT);
    render_loop(&mut gl_context, &mut window_data, draw_debug_quadtree, quadtree);
}