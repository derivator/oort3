use super::glutil;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use log::warn;
use nalgebra::{storage::ContiguousStorage, Matrix4, Point2};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

const MAX_VERTICES: i32 = 1024;
const FLOATS_PER_VERTEX: i32 = 6;
const VERTEX_ATTRIB_SIZE: i32 = FLOATS_PER_VERTEX * 4;

pub struct TrailRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer: WebGlBuffer,
    index: i32,
    last_positions: HashMap<ShipHandle, Point2<f32>>,
}

impl TrailRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 color;
out vec4 varying_color;
void main() {
    gl_Position = transform * vertex;
    varying_color = color;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
in vec4 varying_color;
out vec4 fragmentColor;
void main() {
    fragmentColor = varying_color;
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let buffer = context.create_buffer().ok_or("failed to create buffer")?;
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        context.buffer_data_with_i32(
            gl::ARRAY_BUFFER,
            MAX_VERTICES * VERTEX_ATTRIB_SIZE,
            gl::DYNAMIC_DRAW,
        );

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context,
            program,
            transform_loc,
            projection_matrix: Matrix4::identity(),
            buffer,
            index: 0,
            last_positions: HashMap::new(),
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn update(&mut self, sim: &Simulation) {
        let mut data = vec![];
        data.reserve(sim.ships.len() * 2 * FLOATS_PER_VERTEX as usize);
        let mut n = 0;
        for &handle in sim.ships.iter() {
            let ship = sim.ship(handle);
            let color = super::ShipRenderer::team_color(ship.data().team);
            let current_position: Point2<f32> = ship.position().vector.cast::<f32>().into();
            {
                use std::collections::hash_map::Entry;
                match self.last_positions.entry(handle) {
                    Entry::Occupied(mut e) => {
                        let last_position = e.insert(current_position);
                        data.push(last_position.x);
                        data.push(last_position.y);
                        data.push(color.x);
                        data.push(color.y);
                        data.push(color.z);
                        data.push(color.w);

                        data.push(current_position.x);
                        data.push(current_position.y);
                        data.push(color.x);
                        data.push(color.y);
                        data.push(color.z);
                        data.push(color.w);

                        n += 2;
                    }
                    Entry::Vacant(e) => {
                        e.insert(current_position);
                    }
                };
            }
        }

        assert_eq!(n % 2, 0);

        if n == 0 {
            return;
        } else if n > MAX_VERTICES {
            warn!("too many trail vertices ({})", n);
            return;
        }

        if self.index + n > MAX_VERTICES {
            let (data0, data1) = data
                .as_slice()
                .split_at(((MAX_VERTICES - self.index) * FLOATS_PER_VERTEX) as usize);
            self.write_data(self.index, data0);
            self.write_data(0, data1);
            self.index = (self.index + n) % MAX_VERTICES;
        } else {
            self.write_data(self.index, &data);
            self.index += n;
        }

        if self.index == MAX_VERTICES {
            self.index = 0;
        }

        assert!(self.index >= 0);
        assert!(self.index < MAX_VERTICES);
    }

    fn write_data(&mut self, index: i32, data: &[f32]) {
        assert!(!data.is_empty());
        assert!(
            (index * VERTEX_ATTRIB_SIZE) + (data.len() as i32 * 4)
                <= (MAX_VERTICES * VERTEX_ATTRIB_SIZE)
        );

        self.context
            .bind_buffer(gl::ARRAY_BUFFER, Some(&self.buffer));

        unsafe {
            // Note that `Float32Array::view` is somewhat dangerous (hence the
            // `unsafe`!). This is creating a raw view into our module's
            // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
            // (aka do a memory allocation in Rust) it'll cause the buffer to change,
            // causing the `Float32Array` to be invalid.
            //
            // As a result, after `Float32Array::view` we have to be very careful not to
            // do any memory allocations before it's dropped.
            let view = js_sys::Float32Array::view(data);
            self.context.buffer_sub_data_with_i32_and_array_buffer_view(
                /*target=*/ gl::ARRAY_BUFFER,
                /*offset=*/ index * VERTEX_ATTRIB_SIZE,
                /*src_data=*/ &view,
            );
        }
    }

    pub fn draw(&mut self) {
        self.context.use_program(Some(&self.program));

        self.context
            .bind_buffer(gl::ARRAY_BUFFER, Some(&self.buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ VERTEX_ATTRIB_SIZE,
            /*offset=*/ 0,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 1,
            /*size=*/ 4,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ VERTEX_ATTRIB_SIZE,
            /*offset=*/ 8,
        );
        self.context.enable_vertex_attrib_array(1);

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.line_width(1.0);

        self.context.draw_arrays(gl::LINES, 0, MAX_VERTICES);

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
    }
}