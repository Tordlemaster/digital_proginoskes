//When doing raytracing:
//Keep track of solid angle for each pixel's ray
//If triangle in quadtree is smaller than solid angle, take the average of all the star colors in the triangle and use that as color
//Does this mean maybe the quadtree should be constructed from scratch every launch with the solid angle of the pixels at the set
//resolution in mind? To use as a depth limit for the tree and remove the requirement for the fragment shader to do averages of an
//unbounded number of stars every frame

//Stars themselves have a solid angle (at least a perceptual one based on naked eye fov, that is invariant(doesn't change rel. to screen) to zoom level)

use glfw::{fail_on_errors, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};

mod utils;

const SCR_WIDTH: u32 = 1920;
const SCR_HEIGHT: u32 = 1080;

struct WindowData {
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>
}

fn draw_debug_quadtree() {

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

fn render_loop(gl_context: &mut Glfw, wd: &mut WindowData, render_draw: fn()) {
    while !wd.window.should_close() {

        render_draw();

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
        wd.window.swap_buffers();
    }
}

pub fn render_main() {
    let mut gl_context = glfw::init(fail_on_errors!()).unwrap();
    let mut window_data = render_setup(&mut gl_context, SCR_WIDTH, SCR_HEIGHT);
    render_loop(&mut gl_context, &mut window_data, draw_debug_quadtree);
}