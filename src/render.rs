//When doing raytracing:
//Keep track of solid angle for each pixel's ray
//If triangle in quadtree is smaller than solid angle, take the average of all the star colors in the triangle and use that as color
//Does this mean maybe the quadtree should be constructed from scratch every launch with the solid angle of the pixels at the set
//resolution in mind? To use as a depth limit for the tree and remove the requirement for the fragment shader to do averages of an
//unbounded number of stars every frame

//Stars themselves have a solid angle (at least a perceptual one based on naked eye fov, that is invariant(doesn't change rel. to screen) to zoom level)

use std::{cmp::max, f32::consts::PI, ffi::{c_void, CString}, num, ptr::null, time::Duration, u64};

use glam::{Quat, Vec3};
use glfw::{fail_on_errors, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use spin_sleep::sleep;
use utils::{DEBUG_QT_UNI_DEPTH, DEBUG_QT_UNI_STARS, DEBUG_QT_UNI_TRANS};
use crate::{render::utils::{deg_ams, load_blackbody_table, load_shader, load_shader_program, ra_dec_to_xyz}, spherical_quadtree::{SphQtNode, SphQtRoot, StarData}};

mod utils;

const SCR_WIDTH: u32 = 3840;
const SCR_HEIGHT: u32 = 2160;

struct WindowData {
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>
}

fn draw_debug_quadtree_recur(parent: &Box<SphQtNode>, recursive_transformation: &glam::Mat3, max_side_stars: u64, depth: i32, max_depth: i32) {
    //Transformation matrix should be passed down and applied and modified recursively TODO TODO TODO TODO
    //let edge_len = ((parent.corners[1][0] - parent.corners[0][0]) / 2.0).abs();
    let edge_len = 1.0 / (depth + 1) as f32;
    let edge_len = (parent.corners[1][0] - parent.corners[0][0]).abs() / 2.0;
    let new_transformation =  glam::Mat3::from_translation(glam::vec2((parent.corners[0][0] + 1.0) / 2.0, (parent.corners[0][1] + 1.0) / 2.0)) * glam::Mat3::from_scale(glam::vec2(edge_len, edge_len));
    //let scale = glam::Mat3::from_scale(glam::vec2(edge_len, edge_len));
    //let translation  = glam::Mat3::from_translation(glam::vec2((parent.corners[1][0] + 1.0) / 2.0, (parent.corners[1][1] + 1.0) / 2.0));
    //let mut transformation = scale * translation * (*recursive_transformation);
    //let mut transformation = translation * (*recursive_transformation);
    let transformation = *recursive_transformation * new_transformation;
    //let self_transformation = transformation * glam::Mat3::from_scale(glam::vec2(2.0, 2.0)) * glam::Mat3::from_translation(glam::vec2(-0.5, -0.5));

    //println!("Layer {}, Position {} {}, Scale {}", depth, (parent.corners[0][0] + 1.0) / 2.0, (parent.corners[0][1] + 1.0) / 2.0, edge_len);
    //println!("Corners: {:?} {:?}, Midpoint {:?}", parent.corners[0], parent.corners[1], parent.midpoint);
    //println!("Matrix: {}", transformation);
    //println!("New matrix: {}", new_transformation);
    //println!("Self Matrix: {}", transformation);
    //Draw parent
    let area = edge_len * edge_len;

    let mut stars_not_in_children = parent.stars_in_children;
    let mut child_count = 0;

    if depth < max_depth {
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

    let star_density = (stars_not_in_children as f32 / max_side_stars as f32) / (area - ((area / 4.0) * child_count as f32));
    //println!("{} stars, {} stars_not_in_children, {} max_side_stars, {} area, {} star density, {} child_count\n", parent.stars_in_children, stars_not_in_children, max_side_stars, area, star_density, child_count);

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
    let mut i = 0;
    if depth < max_depth {
        for c in &parent.children {
            if c.is_some() {
                let c_uw = c.as_ref().unwrap();

                //Recur into the child
                //if i == 0 || i==1 || i==2 {
                draw_debug_quadtree_recur(c_uw, recursive_transformation, max_side_stars, depth + 1, max_depth);
                //}
            }
            i+=1;
        }
    }
}

fn draw_debug_quadtree(quadtree: &SphQtRoot, max_depth: i32) {

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
    
    for i in 0..6 {
        //Depth first search
        //Go deep until reach Option<None>
        let side = &quadtree.faces[i];

        let mut transformation = glam::Mat3::from_translation(base_positions[i]) * glam::Mat3::from_scale(base_scale);
        transformation = glam::Mat3::from_scale(glam::vec2(SCR_HEIGHT as f32 / SCR_WIDTH as f32, 1.0)) * glam::Mat3::from_scale(glam::vec2(2.0, 2.0)) * glam::Mat3::from_translation(glam::vec2(-0.5, -0.5)) * transformation;
        //transformation *= glam::Mat3::from_translation(base_positions[i]);
        //transformation *= glam::Mat3::from_scale(glam::vec2(SCR_HEIGHT as f32 / SCR_WIDTH as f32, 1.0)); //Screen resolution

        //transformation = glam::Mat3::IDENTITY;

        draw_debug_quadtree_recur(side.as_ref().unwrap(), &transformation, max_side_stars, 0, max_depth);
    }

    

    /*unsafe {
        gl::UseProgram(utils::DEBUG_QT_PROGRAM);
        gl::BindVertexArray(utils::QUAD_VAO);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        gl::BindVertexArray(0);
    }*/
}

fn star_list(quadtree: &SphQtRoot, star_list: &mut Vec<StarData>) {
    for i in 0..6 {
        star_list_recur(&quadtree.faces[i].as_ref().unwrap(), star_list);
    }
}

fn star_list_recur(node: &Box<SphQtNode>, star_list: &mut Vec<StarData>) {
    if node.stars.len() > 0 {
        //Base case
        for s in &node.stars.iter().map(|s| StarData{ra: s.ra.to_radians(), dec: s.dec.to_radians(), bt: s.bt, vt: s.vt}).collect::<Vec<StarData>>() {
            star_list.push(*s);
        }
    }
    else {
        for c in &node.children {
            if let Some(s) = c.as_ref() {
                star_list_recur(s, star_list);
            }
        }
    }
}

fn setup_ursa_minor() -> (u32, u32) {
    let mut um_vao = 0;
    let mut um_vbo = 0;
    let mut um_prog = 0;
    let um_verts = [
        ra_dec_to_xyz(
            ((deg_ams(2.0, 31.0, 49.09) / 24.0) * 360.0).to_radians(),
            deg_ams(89.0, 15.0, 50.8).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(17.0, 32.0, 12.99671) / 24.0) * 360.0).to_radians(),
            deg_ams(86.0, 35.0, 11.2584).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(16.0, 45.0, 58.24168) / 24.0) * 360.0).to_radians(),
            deg_ams(82.0, 2.0, 14.1233).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(15.0, 44.0, 3.51892) / 24.0) * 360.0).to_radians(),
            deg_ams(77.0, 47.0, 40.1788).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(14.0, 50.0, 42.32580) / 24.0) * 360.0).to_radians(),
            deg_ams(74.0, 9.0, 19.8142).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(15.0, 20.0, 43.71604) / 24.0) * 360.0).to_radians(),
            deg_ams(71.0, 50.0, 2.4596).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(16.0, 17.0, 30.27025) / 24.0) * 360.0).to_radians(),
            deg_ams(75.0, 45.0, 19.2351).to_radians()
        ),
        ra_dec_to_xyz(
            ((deg_ams(15.0, 44.0, 3.51892) / 24.0) * 360.0).to_radians(),
            deg_ams(77.0, 47.0, 40.1788).to_radians()
        ),
    ];
    unsafe {
        gl::GenVertexArrays(1, &raw mut um_vao);
        gl::BindVertexArray(um_vao);

        gl::GenBuffers(1, &raw mut um_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, um_vbo);

        gl::BufferData(gl::ARRAY_BUFFER, (size_of_val(&um_verts)).try_into().unwrap(), (&raw const um_verts) as *const c_void, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * size_of::<f32>()).try_into().unwrap(), 0 as *const c_void);

        gl::BindVertexArray(0);

        um_prog = load_shader_program(vec![
            load_shader(gl::VERTEX_SHADER, "./src/render/shaders/constellations.vert"),
            load_shader(gl::FRAGMENT_SHADER, "./src/render/shaders/constellations.frag")
        ]);
    }
    (um_vao, um_prog)
}

fn setup_draw_stars(quadtree: &SphQtRoot) -> (u32, u32, usize, u32, u32) {
    let mut stars = Vec::new();
    star_list(quadtree, &mut stars);

    println!("{:?}", stars[0..3].iter().map(|s| (
        s.dec.sin() * s.ra.cos(),
        s.dec.sin() * s.ra.sin(),
        s.ra.cos()
    )).collect::<Vec<(f32, f32, f32)>>());

    /*let mut star_positions = Vec::new();
    let mut star_colors = Vec::new();

    for s in stars {
        star_positions.push((
            s.dec.sin() * s.ra.cos(),
            s.dec.sin() * s.ra.sin(),
            s.ra.cos()
        ));
        star_colors.push((1.0, 1.0, 1.0));
    }*/

    let mut stars_vao = 0;
    let mut stars_vbo = 0;
    let mut cam_ubo = 0;
    let mut stars_program = 0;
    unsafe {
        println!("Point size");
        gl::Enable(gl::POINT_SIZE);
        gl::PointSize(11.0);

        println!("VAO");
        gl::GenVertexArrays(1, &raw mut stars_vao);
        gl::BindVertexArray(stars_vao);

        println!("VBO");
        gl::GenBuffers(1, &raw mut stars_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, stars_vbo);

        println!("VBO data");
        gl::BufferData(gl::ARRAY_BUFFER, (stars.len() * (6 * size_of::<f32>())).try_into().unwrap(), std::ptr::null(), gl::STATIC_DRAW);
        println!("map named buffer");
        let star_buf = gl::MapNamedBuffer(stars_vbo, gl::WRITE_ONLY);
        println!("write data");
        let bb_table = load_blackbody_table();
        for i in 0..stars.len() {
            let s = &stars[i];
            let s_temp = 7000.0 / (s.bt - s.vt + 0.56);
            let s_irradiance = 10.0_f32.powf(0.4 * (-s.vt - 19.0));
            let s_rgb = bb_table.temp_to_xy(s_temp).to_rgb() * s_irradiance * 1000000000.0;
            let s_xyz = ra_dec_to_xyz(s.ra, s.dec);
            *(star_buf as *mut(f32, f32, f32, f32, f32, f32)).add(i) = (
                s_xyz.0,
                s_xyz.1,
                s_xyz.2,
                s_rgb.x,
                s_rgb.y,
                s_rgb.z
            );
        }
        println!("unmap buffer");
        assert!(gl::UnmapNamedBuffer(stars_vbo) == gl::TRUE);

        println!("Vertex attribs");
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (6 * size_of::<f32>()).try_into().unwrap(), 0 as *const c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (6 * size_of::<f32>()).try_into().unwrap(), (3 * size_of::<f32>()) as *const c_void);
        
        gl::BindVertexArray(0);

        gl::GenBuffers(1, &raw mut cam_ubo);
        gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, cam_ubo);
        gl::BufferData(gl::UNIFORM_BUFFER, (16 * size_of::<f32>()).try_into().unwrap(), null(), gl::DYNAMIC_DRAW);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

        println!("Load shader");
        stars_program = load_shader_program(vec![
            load_shader(gl::VERTEX_SHADER, "./src/render/shaders/stars.vert"),
            load_shader(gl::FRAGMENT_SHADER, "./src/render/shaders/stars.frag")
        ]);
    }
    println!("{}", stars_program);
    (stars_vao, stars_vbo, stars.len(), cam_ubo, stars_program)
}

fn render_setup(gl_context: &mut Glfw, scr_width: u32, scr_height: u32) -> WindowData {
    gl_context.window_hint(glfw::WindowHint::ContextVersion(4, 2));
    gl_context.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    #[cfg(target_os = "macos")] {
        gl_context.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    }

    let (mut window, events) = gl_context.create_window(scr_width, scr_height, "Digital Progonoskes", glfw::WindowMode::Windowed).expect("Failed to create window");
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|s| window.get_proc_address(s));

    //utils::init_utils();
    //utils::setup_debug_qt_program();

    WindowData { window: window, events: events }
}

fn debug_render_loop(gl_context: &mut Glfw, wd: &mut WindowData, render_draw: fn(&SphQtRoot, i32), quadtree: &SphQtRoot) {
    let mut already_drew_debug = false;
    let mut debug_max_draw_depth = 5;
    while !wd.window.should_close() {

        if !already_drew_debug { 
            render_draw(quadtree, debug_max_draw_depth);
            wd.window.swap_buffers();
            already_drew_debug = true;
        }
        //render_draw(quadtree);

        //Take keyboard input
        gl_context.poll_events();
        for (_, event) in glfw::flush_messages(&wd.events) {
            println!("{:?}", event);
              match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) =>
                    wd.window.set_should_close(true),
                glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                    already_drew_debug = false;
                    debug_max_draw_depth += 1;
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                    already_drew_debug = false;
                    debug_max_draw_depth -= 1;
                }
                _ => {}
            }
        }

        //Swap buffers
        //wd.window.swap_buffers();
    }
}

fn stars_render_loop(gl_context: &mut Glfw, wd: &mut WindowData, quadtree: &SphQtRoot) {
    let (stars_vao, stars_vbo, n_stars, cam_ubo, stars_program) = setup_draw_stars(quadtree);
    let (um_vao, um_program) = setup_ursa_minor();
    println!("setup done");
    let n_stars: i32 = n_stars.try_into().unwrap();

    let cam_proj = glam::Mat4::perspective_lh(90.0_f32.to_radians(), 16.0/9.0, 0.001, 100.0);
    let mut cam_az = 0.0;
    let mut cam_ele = 0.0;
    let mut cam_view = glam::Mat4::from_rotation_translation(Quat::from_euler(glam::EulerRot::XYZEx, cam_ele, cam_az, 0.0), Vec3::ZERO);

    unsafe {
        gl::BindVertexArray(stars_vao);
        gl::UseProgram(stars_program);
        gl::Disable(gl::DEPTH_TEST);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
    println!("go, {}", stars_program);
    while !wd.window.should_close() {
        cam_view = glam::Mat4::from_quat(Quat::from_euler(glam::EulerRot::XYZ, cam_ele, cam_az, 0.0));
        let cam_proj_view = cam_proj * cam_view;
        //println!("{}", cam_proj_view);
        unsafe {
            //Update transformation matrix
            //gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, cam_ubo);
            //gl::BufferSubData(gl::UNIFORM_BUFFER, 0, (size_of::<f32>() * 16).try_into().unwrap(), (&raw const cam_proj_view) as *const c_void);
            //gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            let cpv = cam_proj_view.to_cols_array();
            gl::NamedBufferSubData(cam_ubo, 0, (size_of::<f32>() * 16).try_into().unwrap(), (&raw const cpv) as *const c_void);
            
            /*let b = b"cam_proj_view\0";
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(stars_program, "cam_proj_view".as_ptr() as *const i8),
                1,
                gl::FALSE,
                (&raw const cpv) as *const f32
            );*/

            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(stars_vao);
            gl::UseProgram(stars_program);
            gl::DrawArrays(gl::POINTS, 0, n_stars);

            gl::BindVertexArray(um_vao);
            gl::UseProgram(um_program);
            gl::DrawArrays(gl::LINE_STRIP, 0, 8);

            gl_context.poll_events();
            for (_, event) in glfw::flush_messages(&wd.events) {
                println!("{:?}", event);
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) =>
                        wd.window.set_should_close(true),
                    glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                        cam_az += 0.3;
                    }
                    glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                        cam_az -= 0.3;
                    }
                    glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                        cam_ele += 0.3;
                    }
                    glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                        cam_ele -= 0.3;
                    }
                    _ => {}
                }
            }
            cam_ele = cam_ele.clamp(const{-PI/2.0}, const{PI/2.0});
            wd.window.swap_buffers();
            sleep(Duration::from_millis(50));
        }
    }
}

pub fn render_main(quadtree: &SphQtRoot) {
    let mut gl_context = glfw::init(fail_on_errors!()).unwrap();
    let mut window_data = render_setup(&mut gl_context, SCR_WIDTH, SCR_HEIGHT);
    stars_render_loop(&mut gl_context, &mut window_data, quadtree);
}