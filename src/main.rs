#[macro_use]
extern crate argle;
extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate nom;
extern crate rand;
use gfx::Device;

use argle::*;
use std::fmt;
use std::thread;
use std::time::Duration;
use glutin::GlContext;
use rand::{Rng, SeedableRng};

type ColourFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

type Format = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);
type View = <Format as gfx::format::Formatted>::View;

mod colour;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

gfx_vertex_struct!(QuadCorners {
    corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
});

gfx_constant_struct!(LifeProperties {
    size_in_pixels: [f32; 2] = "u_SizeInPixels",
    survive_min: u32 = "u_SurviveMin",
    survive_max: u32 = "u_SurviveMax",
    resurrect_min: u32 = "u_ResurrectMin",
    resurrect_max: u32 = "u_ResurrectMax",
});

gfx_constant_struct!(OutputProperties {
    alive_colour: [f32; 4] = "u_AliveColour",
    dead_colour: [f32; 4] = "u_DeadColour",
});

gfx_constant_struct!(InitProperties {
    seed: f32 = "u_Seed",
    _pad: u32 = "_u_Pad",
    size_in_pixels: [f32; 2] = "u_SizeInPixels",
});

gfx_pipeline!(life_pipe {
    quad_corners: gfx::VertexBuffer<QuadCorners> = (),
    properties: gfx::ConstantBuffer<LifeProperties> = "Properties",
    out_colour: gfx::BlendTarget<ColourFormat> =
        ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    in_colour: gfx::TextureSampler<View> = "t_InColour",
});

gfx_pipeline!(flip_pipe {
    quad_corners: gfx::VertexBuffer<QuadCorners> = (),
    out_colour: gfx::BlendTarget<ColourFormat> =
        ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    in_colour: gfx::TextureSampler<View> = "t_InColour",
});

gfx_pipeline!(init_pipe {
    quad_corners: gfx::VertexBuffer<QuadCorners> = (),
    properties: gfx::ConstantBuffer<InitProperties> = "Properties",
    out_colour: gfx::BlendTarget<ColourFormat> =
        ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
});

gfx_pipeline!(output_pipe {
    quad_corners: gfx::VertexBuffer<QuadCorners> = (),
    properties: gfx::ConstantBuffer<OutputProperties> = "Properties",
    out_colour: gfx::BlendTarget<ColourFormat> =
        ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    in_colour: gfx::TextureSampler<View> = "t_InColour",
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

fn life_bundle<F, R>(
    in_colour: gfx::handle::ShaderResourceView<R, View>,
    out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
    factory: &mut F,
) -> gfx::Bundle<R, life_pipe::Data<R>>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let sampler = common_sampler(factory);
    let (quad_corners_buf, slice) = common_buffers(factory);
    let data = life_pipe::Data {
        quad_corners: quad_corners_buf,
        properties: factory.create_constant_buffer(1),
        out_colour: out_colour.clone(),
        in_colour: (in_colour, sampler),
    };
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/life.150.vert"),
            include_bytes!("shaders/life.150.frag"),
            life_pipe::new(),
        )
        .expect("Failed to create pipeline");
    gfx::pso::bundle::Bundle::new(slice, pso, data)
}

fn flip_bundle<F, R>(
    in_colour: gfx::handle::ShaderResourceView<R, View>,
    out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
    factory: &mut F,
) -> gfx::Bundle<R, flip_pipe::Data<R>>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let sampler = common_sampler(factory);
    let (quad_corners_buf, slice) = common_buffers(factory);
    let data = flip_pipe::Data {
        quad_corners: quad_corners_buf,
        out_colour: out_colour,
        in_colour: (in_colour, sampler),
    };
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/flip.150.vert"),
            include_bytes!("shaders/flip.150.frag"),
            flip_pipe::new(),
        )
        .expect("Failed to create pipeline");
    gfx::pso::bundle::Bundle::new(slice, pso, data)
}

fn init_bundle<F, R>(
    out_colour: gfx::handle::RenderTargetView<R, ColourFormat>,
    factory: &mut F,
) -> gfx::Bundle<R, init_pipe::Data<R>>
where
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    R: gfx::Resources,
{
    let (quad_corners_buf, slice) = common_buffers(factory);
    let data = init_pipe::Data {
        quad_corners: quad_corners_buf,
        properties: factory.create_constant_buffer(1),
        out_colour: out_colour,
    };
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/init.150.vert"),
            include_bytes!("shaders/init.150.frag"),
            init_pipe::new(),
        )
        .expect("Failed to create pipeline");
    gfx::pso::bundle::Bundle::new(slice, pso, data)
}

fn output_bundle<F, R>(
    in_colour: gfx::handle::ShaderResourceView<R, View>,
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
        out_colour: out_colour,
        in_colour: (in_colour, sampler),
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
    life_width: u32,
    life_height: u32,
    life: gfx::Bundle<R, life_pipe::Data<R>>,
    flip: gfx::Bundle<R, flip_pipe::Data<R>>,
    init: gfx::Bundle<R, init_pipe::Data<R>>,
    output: gfx::Bundle<R, output_pipe::Data<R>>,
}

impl<R: gfx::Resources> Renderer<R> {
    fn new<F>(
        width: u32,
        height: u32,
        cell_size: u32,
        rtv: gfx::handle::RenderTargetView<R, ColourFormat>,
        factory: &mut F,
    ) -> Self
    where
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    {
        let life_width = width / cell_size;
        let life_height = height / cell_size;
        let (_, life_in, flip_out) = factory
            .create_render_target(life_width as u16, life_height as u16)
            .expect("Failed to create render target");
        let (_, flip_in, life_out) = factory
            .create_render_target(life_width as u16, life_height as u16)
            .expect("Failed to create render target");
        let init = init_bundle(flip_out.clone(), factory);
        let life = life_bundle(life_in, life_out, factory);
        let flip = flip_bundle(flip_in.clone(), flip_out, factory);
        let output = output_bundle(flip_in, rtv, factory);
        Self {
            life_width,
            life_height,
            init,
            life,
            flip,
            output,
        }
    }

    fn init<C>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        rng: &mut impl Rng,
        colours: Colours,
        GameParams {
            survive_min,
            survive_max,
            resurrect_min,
            resurrect_max,
        }: GameParams,
    ) where
        C: gfx::CommandBuffer<R>,
    {
        let size_in_pixels = [self.life_width as f32, self.life_height as f32];
        encoder.update_constant_buffer(
            &self.init.data.properties,
            &InitProperties {
                seed: rng.gen(),
                _pad: 0,
                size_in_pixels,
            },
        );
        self.init.encode(encoder);
        encoder.update_constant_buffer(
            &self.life.data.properties,
            &LifeProperties {
                size_in_pixels,
                survive_min,
                survive_max,
                resurrect_min,
                resurrect_max,
            },
        );
        encoder.update_constant_buffer(
            &self.output.data.properties,
            &OutputProperties {
                alive_colour: colours.alive,
                dead_colour: colours.dead,
            },
        );
    }

    fn draw<C>(&self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        self.life.encode(encoder);
        self.flip.encode(encoder);
        self.output.encode(encoder);
    }
}

fn run(
    Args {
        mut rng,
        window_size,
        cell_size,
        colours,
        game_params,
        delay,
    }: Args,
) {
    let mut events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new().with_title("life-gl").with_resizable(true);
    let builder = match window_size {
        WindowSize::Fullscreen => {
            let primary_monitor = events_loop.get_primary_monitor();
            builder.with_fullscreen(Some(primary_monitor))
        }
        WindowSize::Dimensions(width, height) => {
            let size = glutin::dpi::LogicalSize::new(width, height);
            builder
                .with_dimensions(size)
        }
    };
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let (window, mut device, mut factory, rtv, _dsv) =
        gfx_window_glutin::init::<_, DepthFormat>(builder, context, &events_loop);
    let glutin::dpi::LogicalSize { width, height } = window.get_inner_size().unwrap();
    let mut encoder: gfx::Encoder<_, gfx_device_gl::CommandBuffer> =
        factory.create_command_buffer().into();
    let mut renderer = Renderer::new(
        width as u32,
        height as u32,
        cell_size as u32,
        rtv,
        &mut factory,
    );
    renderer.init(&mut encoder, &mut rng, colours, game_params);
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
                                glutin::VirtualKeyCode::Space => {
                                    renderer.init(&mut encoder, &mut rng, colours, game_params);
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                glutin::WindowEvent::Resized(size) => {
                    let (rtv, _) = gfx_window_glutin::new_views::<_, DepthFormat>(&window);
                    renderer = Renderer::new(
                        size.width as u32,
                        size.height as u32,
                        cell_size as u32,
                        rtv,
                        &mut factory,
                    );
                    renderer.init(&mut encoder, &mut rng, colours, game_params);
                }
                _ => (),
            },
            _ => (),
        });

        if !running {
            break;
        }

        renderer.draw(&mut encoder);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        if let Some(delay) = delay {
            thread::sleep(delay);
        }
    }
}

const DEFAULT_WIDTH: f64 = 640.;
const DEFAULT_HEIGHT: f64 = 480.;

#[derive(Clone, Debug)]
enum WindowSize {
    Fullscreen,
    Dimensions(f64, f64),
}

impl fmt::Display for WindowSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl WindowSize {
    fn params() -> impl Param<Item = Self> {
        let dimensions = codepend_params! {
            arg_opt("x", "width", "width of window in pixels", "FLOAT"),
            arg_opt("y", "height", "height of window in pixels", "FLOAT"),
        }.opt_map(|(width, height)| WindowSize::Dimensions(width, height));
        let fullscreen =
            flag("f", "fullscreen", "take up the entire screen").some_if(WindowSize::Fullscreen);
        dimensions
            .either_homogeneous(fullscreen)
            .with_default(WindowSize::Dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT))
    }
}

#[derive(Clone, Copy)]
struct Colours {
    alive: [f32; 4],
    dead: [f32; 4],
}

impl Colours {
    fn params() -> impl argle::Param<Item = Self> {
        map_params! {
            let {
                alive =
                    arg_opt_def("a", "alive-colour", "colour of alive cells in hex", "#RRGGBB",
                                "#FFFFFF".to_string()).convert(|s| colour::parse_colour(s));
                dead =
                    arg_opt_def("d", "dead-colour", "colour of alive cells in hex", "#RRGGBB",
                                "#000000".to_string()).convert(|s| colour::parse_colour(s));
            } in {
                Self { alive, dead }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct GameParams {
    survive_min: u32,
    survive_max: u32,
    resurrect_min: u32,
    resurrect_max: u32,
}

const DEFAULT_SURVIVE_MIN: u32 = 2;
const DEFAULT_SURVIVE_MAX: u32 = 3;
const DEFAULT_RESURRECT_MIN: u32 = 3;
const DEFAULT_RESURRECT_MAX: u32 = 3;

impl GameParams {
    fn params() -> impl Param<Item = Self> {
        map_params! {
            let {
                survive_min = arg_opt_def(
                    "s",
                    "survive-min",
                    "minimum living neighbours to survive",
                    "INT",
                    DEFAULT_SURVIVE_MIN
                );
                survive_max = arg_opt_def(
                    "t",
                    "survive-max",
                    "maximum living neighbours to survive",
                    "INT",
                    DEFAULT_SURVIVE_MAX
                );
                resurrect_min = arg_opt_def(
                    "r",
                    "resurrect-min",
                    "minimum living neighbours to resurrect",
                    "INT",
                    DEFAULT_RESURRECT_MIN
                );
                resurrect_max = arg_opt_def(
                    "u",
                    "resurrect-max",
                    "maximum living neighbours to resurrect",
                    "INT",
                    DEFAULT_RESURRECT_MAX
                );
            } in {
                Self {
                    survive_min,
                    survive_max,
                    resurrect_min,
                    resurrect_max,
                }
            }
        }
    }
}

struct Args {
    rng: rand::StdRng,
    window_size: WindowSize,
    cell_size: u32,
    colours: Colours,
    game_params: GameParams,
    delay: Option<Duration>,
}

impl Args {
    fn params() -> impl argle::Param<Item = Self> {
        map_params! {
            let {
                cell_size = arg_opt_def("c", "cell-size", "size of cell in pixels", "INT", 1);
                colours = Colours::params();
                delay = arg_opt("e", "delay", "delay in ms to pause between frames", "INT")
                    .map(|d| if d == Some(0) { None } else { d })
                    .opt_map(Duration::from_millis);
                rng = arg_opt::<u64>("n", "seed",
                                     "seed for the random number generator (omit for random seed)",
                                     "INT")
                    .map(|seed| match seed {
                        Some(seed) => {
                            let mut buf = [0; 32];
                            for i in 0..8 {
                                buf[i] = ((seed >> i) & 0xff) as u8;
                            }
                            rand::StdRng::from_seed(buf)
                        }
                        None => rand::StdRng::from_rng(rand::thread_rng()).unwrap(),
                    });
                game_params = GameParams::params();
                window_size = WindowSize::params();
            } in {
                Self { cell_size, colours, delay, game_params, rng, window_size }
            }
        }
    }
}

fn main() {
    match Args::params().with_default_help().parse_env_def() {
        (Ok(HelpOr::Value(args)), _usage) => run(args),
        (Ok(HelpOr::Help), usage) => print!("{}", usage.render()),
        (Err(error), usage) => {
            print!("{}\n\n", error);
            print!("{}", usage.render());
            ::std::process::exit(1);
        }
    }
}
