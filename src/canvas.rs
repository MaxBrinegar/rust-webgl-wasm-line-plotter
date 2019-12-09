use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};
use std::collections::HashMap;

pub struct Canvas {
  pub canvas: web_sys::HtmlCanvasElement,
  context: WebGlRenderingContext,
  color_uniform_location: web_sys::WebGlUniformLocation,
  pub width: f32,
  pub height: f32,
  translation_uniform_location: web_sys::WebGlUniformLocation,
  rotation_uniform_location: web_sys::WebGlUniformLocation,
  colors: HashMap<String, (f32, f32, f32, f32)>,
}

impl Canvas {
  pub fn new(id: &str, width: f32, height: f32) -> Canvas {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let context = canvas
        .get_context("webgl")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    let vert_shader = Canvas::compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
          r#"
          attribute vec2 a_position;
          uniform vec2 u_resolution;
          uniform vec2 u_translation;
          uniform vec2 u_rotation;
          void main() {
              // Rotate the position
              vec2 rotatedPosition = vec2(
                 a_position.x * u_rotation.y + a_position.y * u_rotation.x,
                 a_position.y * u_rotation.y - a_position.x * u_rotation.x);
              vec2 position = rotatedPosition + u_translation;
          
              // convert the rectangle from pixels to 0.0 to 1.0
              vec2 zeroToOne = position / u_resolution;
           
              // convert from 0->1 to 0->2
              vec2 zeroToTwo = zeroToOne * 2.0;
           
              // convert from 0->2 to -1->+1 (clip space)
              vec2 clipSpace = zeroToTwo - 1.0;
           
              gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
          }
      "#,
    ).unwrap();
    let frag_shader = Canvas::compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
          r#"
          precision mediump float;
          uniform vec4 u_color;
          void main() {
              gl_FragColor = u_color;
          }
      "#,
    ).unwrap();
    let program = Canvas::link_program(&context, &vert_shader, &frag_shader).unwrap();
    context.use_program(Some(&program));
  
    let buffer = context.create_buffer().ok_or("failed to create buffer").unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
  
    context.vertex_attrib_pointer_with_i32(0, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(0);
  
    context.clear_color(0.0, 0.0, 0.0, 0.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
  
    let resolution_uniform_location = context.get_uniform_location(&program, "u_resolution").unwrap();
    context.uniform2f(Some(&resolution_uniform_location), width, height);

    let translation_uniform_location = context.get_uniform_location(&program, "u_translation").unwrap();

    let color_uniform_location = context.get_uniform_location(&program, "u_color").unwrap();
  
    let mut colors = HashMap::new();
    colors.insert(String::from("red"), (1.0, 0.0, 0.0, 1.0));

    let rotation_uniform_location = context.get_uniform_location(&program, "u_rotation").unwrap();

    Canvas {
      canvas,
      context,
      width,
      height,
      translation_uniform_location,
      rotation_uniform_location,
      color_uniform_location,
      colors,
    }
  }

  pub fn draw_line(
    &self,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: &str
  ) {
    let radians = (y2 - y1).atan2(x2 - x1);
    let cos = radians.cos();
    let sin = radians.sin();
    let distance = ((y2 - y1).powf(2.0) + (x2 - x1).powf(2.0)).sqrt();

    self.draw_rectangle(x1, y1, distance, 5.0, cos, sin, color);
  }

  pub fn draw_rectangle(
    &self,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rot_x: f32,
    rot_y: f32,
    color: &str,
  ) {
    let color = match self.colors.get(color) {
      Some(&tuple) => tuple,
      // default to black
      _ => (0.0, 0.0, 0.0, 1.0),
    };

    Canvas::set_rectangle(&self.context, width, height);
    self.context.uniform4f(Some(&self.color_uniform_location), color.0, color.1, color.2, color.3);
    self.context.uniform2f(Some(&self.translation_uniform_location), x, y);
    self.context.uniform2f(Some(&self.rotation_uniform_location), rot_x, rot_y);
    self.context.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
  }

  fn set_rectangle(context: &WebGlRenderingContext,
    width: f32,
    height: f32
  ) {
    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vert_array = js_sys::Float32Array::view(&[
          0.0, 0.0,
          height, 0.0,
          0.0, width,
          0.0, width,
          height, 0.0,
          height, width,
      ]);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
  }

  fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
  ) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
  }

  fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
  ) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
  }
}