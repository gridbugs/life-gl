extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
use gfx::Device;
use gfx::Factory;

use glutin::GlContext;

type ColourFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

type Format = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);
type View = <Format as gfx::format::Formatted>::View;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

gfx_vertex_struct!(QuadCorners {
    corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
});

gfx_constant_struct!(InternalProperties {
    init: u32 = "u_Init",
    delta: f32 = "u_Delta",
    size_in_pixels: [f32; 2] = "u_SizeInPixels",
});

gfx_constant_struct!(OutputProperties {
    which_input_to_render_from: u32 = "u_WhichInputToRenderFrom",
});

gfx_pipeline!(internal_pipe {
        quad_corners: gfx::VertexBuffer<QuadCorners> = (),
        properties: gfx::ConstantBuffer<InternalProperties> = "Properties",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        in_colour: gfx::TextureSampler<View> = "t_InColour",
});

gfx_pipeline!(output_pipe {
        quad_corners: gfx::VertexBuffer<QuadCorners> = (),
        properties: gfx::ConstantBuffer<OutputProperties> = "Properties",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        in_colour0: gfx::TextureSampler<View> = "t_InColour0",
        in_colour1: gfx::TextureSampler<View> = "t_InColour1",
});

fn common_buffers<R, F>(factory: &mut F) -> (gfx::handle::Buffer<R, QuadCorners>, gfx::Slice<R>)
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let quad_corners_data = QUAD_COORDS
        .iter()
        .map(|v| QuadCorners {
            corner_zero_to_one: *v,
        })
        .collect::<Vec<_>>();

    factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..])
}
fn common_sampler<R, F>(factory: &mut F) -> gfx::handle::Sampler<R>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    factory.create_sampler(gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Scale,
        gfx::texture::WrapMode::Border,
    ))
}

fn internal_bundle<F, R>(
    width: u32,
    height: u32,
    in_colour: gfx::handle::ShaderResourceView<R, View>,
    out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
    factory: &mut F,
) -> gfx::Bundle<R, internal_pipe::Data<R>>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let sampler = common_sampler(factory);
    let (quad_corners_buf, slice) = common_buffers(factory);
    let data = internal_pipe::Data {
        quad_corners: quad_corners_buf,
        properties: factory.create_constant_buffer(1),
        out_colour: out_colour.clone(),
        in_colour: (in_colour, sampler),
    };
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/internal.150.vert"),
            include_bytes!("shaders/internal.150.frag"),
            internal_pipe::new(),
        )
        .expect("Failed to create pipeline");
    gfx::pso::bundle::Bundle::new(slice, pso, data)
}

fn output_bundle<F, R>(
    width: u32,
    height: u32,
    in_colour0: gfx::handle::ShaderResourceView<R, View>,
    in_colour1: gfx::handle::ShaderResourceView<R, View>,
    out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
    factory: &mut F,
) -> gfx::Bundle<R, output_pipe::Data<R>>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let sampler = common_sampler(factory);
    let (quad_corners_buf, slice) = common_buffers(factory);
    let data = output_pipe::Data {
        quad_corners: quad_corners_buf,
        properties: factory.create_constant_buffer(1),
        out_colour: out_colour.clone(),
        in_colour0: (in_colour0, sampler.clone()),
        in_colour1: (in_colour1, sampler),
    };
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/output.150.vert"),
            include_bytes!("shaders/output.150.frag"),
            output_pipe::new(),
        )
        .expect("Failed to create pipeline");
    gfx::pso::bundle::Bundle::new(slice, pso, data)
}

struct Renderer<R: gfx::Resources> {
    width: u32,
    height: u32,
    current: gfx::Bundle<R, internal_pipe::Data<R>>,
    other: gfx::Bundle<R, internal_pipe::Data<R>>,
    output: gfx::Bundle<R, output_pipe::Data<R>>,
    output_properties: OutputProperties,
}

impl<R: gfx::Resources> Renderer<R> {
    fn new<F>(
        width: u32,
        height: u32,
        rtv: gfx::handle::RenderTargetView<R, ColourFormat>,
        factory: &mut F,
    ) -> Self
    where
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    {
        let (_, srv0, rtv0) = factory
            .create_render_target(width as u16, width as u16)
            .expect("Failed to create render target");
        let (_, srv1, rtv1) = factory
            .create_render_target(width as u16, height as u16)
            .expect("Failed to create render target");
        let current = internal_bundle(width, height, srv0.clone(), rtv1, factory);
        let other = internal_bundle(width, height, srv1.clone(), rtv0, factory);
        let output = output_bundle(width, height, srv0, srv1, rtv, factory);
        let output_properties = OutputProperties {
            which_input_to_render_from: 0,
        };
        Self {
            width,
            height,
            current,
            other,
            output,
            output_properties,
        }
    }

    fn init_0<C>(&mut self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        let size_in_pixels = [self.width as f32, self.height as f32];
        encoder.update_constant_buffer(
            &self.other.data.properties,
            &InternalProperties {
                init: 1,
                delta: 0.,
                size_in_pixels,
            },
        );
        self.other.encode(encoder);
    }
    fn init_1<C>(&mut self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        let size_in_pixels = [self.width as f32, self.height as f32];
        encoder.update_constant_buffer(
            &self.current.data.properties,
            &InternalProperties {
                init: 0,
                delta: 1.,
                size_in_pixels,
            },
        );
        self.current.encode(encoder);
    }
    fn init_2<C>(&mut self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        let size_in_pixels = [self.width as f32, self.height as f32];
        encoder.update_constant_buffer(
            &self.other.data.properties,
            &InternalProperties {
                init: 0,
                delta: 0.,
                size_in_pixels,
            },
        );
        self.other.encode(encoder);
    }

    fn draw<C>(&self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        encoder.clear(&self.current.data.out_colour, [0., 0., 0., 1.]);
        self.current.encode(encoder);
        self.output.encode(encoder);
    }

    fn swap_buffers<C>(&mut self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        ::std::mem::swap(&mut self.current, &mut self.other);
        self.output_properties.which_input_to_render_from = 1;
        encoder.update_constant_buffer(&self.output.data.properties, &self.output_properties);
        self.output.encode(encoder);
    }
}

fn main() {
    let window_width = 1920;
    let window_height = 1080;

    let builder = glutin::WindowBuilder::new()
        .with_dimensions(window_width, window_height)
        .with_max_dimensions(window_width, window_height)
        .with_min_dimensions(window_width, window_height);

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let (window, mut device, mut factory, rtv, _dsv) =
        gfx_window_glutin::init::<ColourFormat, DepthFormat>(builder, context, &events_loop);

    let mut encoder: gfx::Encoder<_, gfx_device_gl::CommandBuffer> =
        factory.create_command_buffer().into();

    let mut renderer = Renderer::new(window_width, window_height, rtv, &mut factory);

    renderer.init_0(&mut encoder);
    renderer.init_1(&mut encoder);
    renderer.init_2(&mut encoder);

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                _ => (),
            },
            _ => (),
        });

        if !running {
            break;
        }
        renderer.draw(&mut encoder);
        renderer.swap_buffers(&mut encoder);
        renderer.draw(&mut encoder);
        renderer.swap_buffers(&mut encoder);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
