extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
use gfx::traits::FactoryExt;
use gfx::Device;
use gfx::Factory;

use glutin::GlContext;

type ColourFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;
type Resources = gfx_device_gl::Resources;

type Format = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);
type Surface = <Format as gfx::format::Formatted>::Surface;
type View = <Format as gfx::format::Formatted>::View;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

gfx_vertex_struct!(QuadCorners {
    corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
});

mod internal {
    use super::*;
    gfx_constant_struct!(Properties {
        init: u32 = "u_Init",
        delta: f32 = "u_Delta",
    });

    gfx_pipeline!(pipe {
        quad_corners: gfx::VertexBuffer<QuadCorners> = (),
        properties: gfx::ConstantBuffer<Properties> = "Properties",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        in_colour: gfx::TextureSampler<View> = "t_InColour",
    });

    pub struct Renderer<R: gfx::Resources> {
        pub bundle: gfx::Bundle<R, pipe::Data<R>>,
    }

    impl<R: gfx::Resources> Renderer<R> {
        pub fn new<F>(
            width: u32,
            height: u32,
            in_colour: gfx::handle::ShaderResourceView<R, View>,
            out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
            factory: &mut F,
        ) -> Self
        where
            F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
        {
            let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
                gfx::texture::WrapMode::Border,
            ));

            let quad_corners_data = QUAD_COORDS
                .iter()
                .map(|v| QuadCorners {
                    corner_zero_to_one: *v,
                })
                .collect::<Vec<_>>();

            let (quad_corners_buf, slice) =
                factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);

            let data = pipe::Data {
                quad_corners: quad_corners_buf,
                properties: factory.create_constant_buffer(1),
                out_colour: out_colour.clone(),
                in_colour: (in_colour, sampler),
            };

            let pso = factory
                .create_pipeline_simple(
                    include_bytes!("shaders/common.150.vert"),
                    include_bytes!("shaders/internal.150.frag"),
                    pipe::new(),
                )
                .expect("Failed to create pipeline");

            let bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);

            Self { bundle }
        }
    }
}

mod output {
    use super::*;

    gfx_constant_struct!(Properties {
        which_input_to_render_from: u32 = "u_WhichInputToRenderFrom",
    });

    gfx_pipeline!(pipe {
        quad_corners: gfx::VertexBuffer<QuadCorners> = (),
        properties: gfx::ConstantBuffer<Properties> = "Properties",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        in_colour0: gfx::TextureSampler<View> = "t_InColour0",
        in_colour1: gfx::TextureSampler<View> = "t_InColour1",
    });

    pub struct Renderer<R: gfx::Resources> {
        pub bundle: gfx::Bundle<R, pipe::Data<R>>,
    }

    impl<R: gfx::Resources> Renderer<R> {
        pub fn new<F>(
            width: u32,
            height: u32,
            in_colour0: gfx::handle::ShaderResourceView<R, View>,
            in_colour1: gfx::handle::ShaderResourceView<R, View>,
            out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
            factory: &mut F,
        ) -> Self
        where
            F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
        {
            let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
                gfx::texture::WrapMode::Border,
            ));

            let quad_corners_data = QUAD_COORDS
                .iter()
                .map(|v| QuadCorners {
                    corner_zero_to_one: *v,
                })
                .collect::<Vec<_>>();

            let (quad_corners_buf, slice) =
                factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);

            let data = pipe::Data {
                quad_corners: quad_corners_buf,
                properties: factory.create_constant_buffer(1),
                out_colour: out_colour.clone(),
                in_colour0: (in_colour0, sampler.clone()),
                in_colour1: (in_colour1, sampler),
            };

            let pso = factory
                .create_pipeline_simple(
                    include_bytes!("shaders/common.150.vert"),
                    include_bytes!("shaders/output.150.frag"),
                    pipe::new(),
                )
                .expect("Failed to create pipeline");

            let bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);

            Self { bundle }
        }
    }
}

fn main() {
    let window_width = 960;
    let window_height = 640;

    let builder = glutin::WindowBuilder::new()
        .with_dimensions(window_width, window_height)
        .with_max_dimensions(window_width, window_height)
        .with_min_dimensions(window_width, window_height);

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let (window, mut device, mut factory, rtv, _dsv) =
        gfx_window_glutin::init::<ColourFormat, DepthFormat>(builder, context, &events_loop);

    let (_, srv0, rtv0) = factory
        .create_render_target(window_width as u16, window_height as u16)
        .expect("Failed to create render target");

    let (_, srv1, rtv1) = factory
        .create_render_target(window_width as u16, window_height as u16)
        .expect("Failed to create render target");

    let internal_a = internal::Renderer::new(
        window_width,
        window_height,
        srv0.clone(),
        rtv1.clone(),
        &mut factory,
    );
    let internal_b = internal::Renderer::new(
        window_width,
        window_height,
        srv1.clone(),
        rtv0.clone(),
        &mut factory,
    );

    let output = output::Renderer::new(window_width, window_height, srv0, srv1, rtv, &mut factory);

    let mut encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer> =
        factory.create_command_buffer().into();

    encoder.update_constant_buffer(
        &internal_a.bundle.data.properties,
        &internal::Properties { init: 1, delta: 0. },
    );

    internal_a.bundle.encode(&mut encoder);
    encoder.flush(&mut device);
    window.swap_buffers().unwrap();
    device.cleanup();

    encoder.update_constant_buffer(
        &internal_a.bundle.data.properties,
        &internal::Properties {
            init: 0,
            delta: 0.0,
        },
    );
    encoder.update_constant_buffer(
        &internal_b.bundle.data.properties,
        &internal::Properties {
            init: 0,
            delta: -0.05,
        },
    );

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match input.state {
                            glutin::ElementState::Pressed => match virtual_keycode {
                                glutin::VirtualKeyCode::Up => {
                                    println!("a");
                                    internal_a.bundle.encode(&mut encoder);
                                    encoder.update_constant_buffer(
                                        &output.bundle.data.properties,
                                        &output::Properties {
                                            which_input_to_render_from: 0,
                                        },
                                    );
                                }
                                glutin::VirtualKeyCode::Down => {
                                    println!("b");
                                    internal_b.bundle.encode(&mut encoder);
                                    encoder.update_constant_buffer(
                                        &output.bundle.data.properties,
                                        &output::Properties {
                                            which_input_to_render_from: 1,
                                        },
                                    );
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });

        if !running {
            break;
        }

        output.bundle.encode(&mut encoder);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
