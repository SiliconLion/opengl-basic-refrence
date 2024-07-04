extern crate glfw;
extern crate gl;

use std::mem::size_of;
// include the OpenGL type aliases
use gl::types::*;

use glfw::{Action, Context, Key};
use glfw::WindowHint::OpenGlForwardCompat;

use std::ptr::{null, null_mut};
use core::ffi::CStr;
use std::ffi::c_char;

static mut SCR_WIDTH: i32 = 800;
static mut SCR_HEIGHT: i32 = 600;


const VERTEX_SHADER_SOURCE: &'static CStr =
c"
#version 330 core
layout (location = 0) in vec3 aPos;
void main()
{
   gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}
";

const FRAGMENT_SHADER_SOURCE: &'static CStr =
c"
#version 330 core
out vec4 FragColor;
void main()
{
   FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}
";

fn main() {
    use glfw::fail_on_errors;
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

//these are the necessary options for MacOS compatibility
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // Create a windowed mode window and its OpenGL context
    //unsafe because we read the value of mutable global
    let (mut window, events) = unsafe {
        glfw.create_window(
            SCR_WIDTH as _, SCR_HEIGHT as _,
            "Hello this is window",
            glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window."
            )
    };

    gl::load_with(|s| window.get_proc_address(s));

    unsafe{
        //this seems silly, but the size of the window created may not be the size of the window requested.
        //Particuarly on highDPI devices or retina displays. So we get the actual size first.
        (SCR_WIDTH , SCR_HEIGHT ) = window.get_size();
        gl::Viewport(0, 0, SCR_WIDTH , SCR_HEIGHT);
    }

    // Make the window's context current
    window.make_current();
    window.set_framebuffer_size_callback(framebuffer_size_callback);
    window.set_key_polling(true);

unsafe {

    // build and compile our shader program
    // ------------------------------------
    // vertex shader
    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
    gl::ShaderSource(
        vertex_shader, 1,
        &VERTEX_SHADER_SOURCE.as_ptr() as _,
        null()
    );
    gl::CompileShader(vertex_shader);
    // check for shader compile errors
    let mut success = 1;
    let mut info_log: [c_char; 512] = [0; 512];
    gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
    if success == 0
    {
        gl::GetShaderInfoLog(vertex_shader, 512, null::<GLsizei>() as _, &mut info_log as _);
        println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED");
        for c in info_log {
            print!("{}", c);
        }
        print!("\n");
    }
    // fragment shader
    let mut fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
    gl::ShaderSource(
        fragment_shader, 1,
        &FRAGMENT_SHADER_SOURCE.as_ptr() as _,
        null()
    );
    gl::CompileShader(fragment_shader);
    // check for shader compile errors
    gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
    if success == 0
    {
        gl::GetShaderInfoLog(fragment_shader, 512, null_mut::<GLsizei>(), &mut info_log as _);
        println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED");
        for c in info_log {
            print!("{}", c);
        }
        print!("\n");
    }
    // link shaders
    let mut shader_program = gl::CreateProgram();
    gl::AttachShader(shader_program, vertex_shader);
    gl::AttachShader(shader_program, fragment_shader);
    gl::LinkProgram(shader_program);
    // check for linking errors
    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
    if success == 0 {
        gl::GetProgramInfoLog(shader_program, 512, null_mut::<GLsizei>(), &mut info_log as _);
        println!("ERROR::SHADER::PROGRAM::LINKING_FAILED");
        for c in info_log {
            print!("{}", c);
        }
        print!("\n");
    }
    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(fragment_shader);

    // set up vertex data (and buffer(s)) and configure vertex attributes
    // ------------------------------------------------------------------
    let mut vertices =
    [
         0.5,  0.5, 0.0,  // top right
         0.5, -0.5, 0.0,  // bottom right
        -0.5, -0.5, 0.0,  // bottom left
        -0.5,  0.5, 0.0   // top left
    ];

    //shrink each vertex down by half.
    for v in &mut vertices {
        *v *= 0.05;
    }

    println!("{:?}", vertices);

    let indices =
    [  // note that we start from 0!
        0, 1, 3,  // first Triangle
        1, 2, 3   // second Triangle
    ];
    let mut VAO = 0;
    let mut VBO = 0;
    let mut EBO = 0;
    gl::GenVertexArrays(1, &mut VAO);
    gl::GenBuffers(1, &mut VBO);
    gl::GenBuffers(1, &mut EBO);
    // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
    gl::BindVertexArray(VAO);

    gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
    // gl::BufferData(gl::ARRAY_BUFFER, (size_of::<f32>() * vertices.len() )as _, vertices.as_ptr() as _, gl::STATIC_DRAW);
    gl::BufferData(gl::ARRAY_BUFFER, 12 * 4, vertices.as_ptr() as _, gl::STATIC_DRAW); //12 floats * 4 bytes

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
    // gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (size_of::<i32>() * indices.len() ) as _, indices.as_ptr() as _, gl::STATIC_DRAW);
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 6 * 4 , indices.as_ptr() as _, gl::STATIC_DRAW); //6 ints * 4 bytes

    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * size_of::<f32>() ) as _, null());
    gl::EnableVertexAttribArray(0);

    // note that this is allowed, the call to glVertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);

    // remember: do NOT unbind the EBO while a VAO is active as the bound element buffer object IS stored in the VAO; keep the EBO bound.
    //glBindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

    // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
    // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
    gl::BindVertexArray(0);


    // uncomment this call to draw in wireframe polygons.
    // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

    // render loop
    // -----------
    while !window.should_close()
    {
        // input
        process_input(&mut window);

        // render
        // ------
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // draw our first triangle
        gl::UseProgram(shader_program);
        gl::BindVertexArray(VAO); // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized
        //glDrawArrays(gl::TRIANGLES, 0, 6);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, null() as _);
        // glBindVertexArray(0); // no need to unbind it every time

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    gl::DeleteVertexArrays(1, &VAO);
    gl::DeleteBuffers(1, &VBO);
    gl::DeleteBuffers(1, &EBO);
    gl::DeleteProgram(shader_program);

    // glfw: terminate, clearing all previously allocated GLFW resources.
    // ------------------------------------------------------------------
    // drop(glfw) calls glfw.Terminate();
}
}

// process all input: query GLFW whether relevant keys are pressed/released this frame and react accordingly
// ---------------------------------------------------------------------------------------------------------
fn process_input(window: &mut glfw::Window)
{
    if window.get_key(glfw::Key::Escape) == glfw::Action::Press {
        window.set_should_close(true);
    }
}

// glfw: whenever the window size changed (by OS or user resize) this callback function executes
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_callback(window: &mut glfw::Window, width: i32, height: i32)
{
    unsafe {
        // make sure the viewport matches the new window dimensions; note that width and
        // height will be significantly larger than specified on retina displays.
        gl::Viewport(0, 0, width, height);
    }
}