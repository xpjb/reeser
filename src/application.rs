use glow::*;
use crate::kinput::*;
use crate::kmath::*;
use crate::kimg::*;
use crate::krenderer::*;
use crate::synth::*;
use crate::sound::*;
use glutin::event::{Event, WindowEvent};
use cpal::Stream;
use cpal::traits::*;
use ringbuf::*;
use std::f32::consts::PI;
use crate::filter::*;

pub struct Application {
    gl: glow::Context,
    window: glutin::WindowedContext<glutin::PossiblyCurrent>,

    renderer: KRenderer,
    event_aggregator: EventAggregator,

    pub xres: f32,
    pub yres: f32,

    audio_stream: Stream,

    synth: Synth,
    channel: Producer<SoundMessage>,
}

pub fn load_file(paths: &[&str]) -> String {
    for path in paths {
        if let Ok(s) = std::fs::read_to_string(path) {
            return s
        }
    }
    panic!("couldn't find any of {:?}", paths)
}

impl Application {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Application {
        let default_xres = 1600.0;
        let default_yres = 1600.0;

        let (gl, window) = unsafe { opengl_boilerplate(default_xres, default_yres, event_loop) };
        
        let uvv = &[
            "src/uv.vert",
            "../../src/uv.vert",
            "uv.vert",
        ];
        let uvf = &[
            "src/uv.frag",
            "../../src/uv.frag",
            "uv.frag",
        ];
                
        let uv_shader = make_shader(&gl, uvv, uvf);
        check_gl_errors(&gl, "after make_shader");

        let atlas = ImageBufferA::new_from_file("src/atlas.png")
            .or(ImageBufferA::new_from_file("../../src/atlas.png")
            .or(ImageBufferA::new_from_file("atlas.png")))
            .expect("couldn't load atlas from ./atlas.png");

        let renderer = KRenderer::new(&gl, uv_shader, atlas);
        check_gl_errors(&gl, "after KRenderer::new");

        let rb = RingBuffer::<SoundMessage>::new(5);
        let (mut prod, mut cons) = rb.split();

        let app = Application {
            gl,
            window,
            renderer,
            event_aggregator: EventAggregator::new(default_xres, default_yres),

            synth: Synth::new(),

            xres: default_xres,
            yres: default_yres,

            channel: prod,
            audio_stream: stream_setup_for(sample_next, cons).expect("no can make stream"),
        };
        app.audio_stream.play().expect("no can play stream");
        app
    }

    pub fn handle_event(&mut self, event: &glutin::event::Event<()>) {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.window.resize(*physical_size);
                    self.xres = physical_size.width as f32;
                    self.yres = physical_size.height as f32;
                    unsafe {self.gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32)};
                },
                _ => {},
            _ => {},
            }
            _ => {},
        }

        if let Some(inputs) = self.event_aggregator.handle_event(event) {
            
            unsafe {
                self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT); 
            } 

            let mut kc = KRCanvas::new();

            self.synth.frame(&inputs, &mut kc, &mut self.channel);

            self.renderer.send(&self.gl, &kc.bytes());

            self.window.swap_buffers().unwrap();
        }
    }

    pub fn destroy(&mut self) {
        self.renderer.destroy(&self.gl);
    }
}

fn check_gl_errors(gl: &glow::Context, label: &str) {
    unsafe {
        loop {
            let err = gl.get_error();
            if err == glow::NO_ERROR {
                break;
            }
            let err_str = match err {
                glow::INVALID_ENUM => "INVALID_ENUM",
                glow::INVALID_VALUE => "INVALID_VALUE",
                glow::INVALID_OPERATION => "INVALID_OPERATION",
                glow::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
                glow::OUT_OF_MEMORY => "OUT_OF_MEMORY",
                _ => "UNKNOWN",
            };
            eprintln!("[GL error] {}: {} (0x{:x})", label, err_str, err);
        }
    }
}

fn  make_shader(gl: &glow::Context, vert_paths: &[&str], frag_paths: &[&str]) -> glow::Program {
    unsafe {
        let program = gl.create_program().expect("Cannot create program");
        let shader_version = "#version 410";
        let shader_sources = [
            (glow::VERTEX_SHADER, load_file(vert_paths)),
            (glow::FRAGMENT_SHADER, load_file(frag_paths)),
        ];
        let mut shaders = Vec::with_capacity(shader_sources.len());
        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        
        program
    }
}

unsafe fn opengl_boilerplate(xres: f32, yres: f32, event_loop: &glutin::event_loop::EventLoop<()>) -> (glow::Context, glutin::WindowedContext<glutin::PossiblyCurrent>) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("synth")
        .with_inner_size(glutin::dpi::PhysicalSize::new(xres, yres));
    let window = glutin::ContextBuilder::new()
        // .with_depth_buffer(0)
        // .with_srgb(true)
        // .with_stencil_buffer(0)
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap()
        .make_current()
        .unwrap();


    let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

    // Debug: print GL info on startup
    unsafe {
        let version = gl.get_parameter_string(VERSION);
        let renderer = gl.get_parameter_string(RENDERER);
        let vendor = gl.get_parameter_string(VENDOR);
        println!("[GL] version: {}", version);
        println!("[GL] renderer: {}", renderer);
        println!("[GL] vendor: {}", vendor);
    }

    gl.enable(DEPTH_TEST);
    // gl.enable(CULL_FACE);
    gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
    gl.enable(BLEND);

    // Enable debug output so callback receives messages
    unsafe {
        gl.enable(DEBUG_OUTPUT);
        gl.debug_message_callback(|_source, _typ, _id, _severity, msg| {
            println!("[GL debug] {}", msg);
        });
    }
    check_gl_errors(&gl, "after opengl_boilerplate");

    (gl, window)
}

fn sample_next(o: &mut SampleRequestOptions) -> f32 {
    o.mixer.tick()
    // o.filter.tick()

    // let distorted = pa_fuzz(samp);
    // let distorted = samp;
    // 0.1 * o.filter.tick(distorted)
    // 0.1 * distorted
    // samp
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub nchannels: usize,

    // pub filter: Filter,

    pub mixer: Mixer,

    pub channel: Consumer<SoundMessage>,
}

pub fn stream_setup_for<F>(on_sample: F, channel: Consumer<SoundMessage>) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample, channel),
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample, channel),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample, channel),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
    channel: Consumer<SoundMessage>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions {
        sample_rate,
        nchannels,

        // sound: Sound::new(),

        // 44100 samples / sec * 0.02
        // o no negative part tho
        // todo lpf
        // use 2 fibonacci numbers
        // maybe fliped distortion
        // sub
        // envelope?
        // keyboard keys & spectrum would be good
        // num detunes, detune amount moving around would be cool. sawtooth fmod lol
        // or 1 kb keys for notes and another for detune. eh it can be a slider.
        // get some reusable components hey. synth gui: kb keys sounds reusable. draw to rect etc
        // yea get the filter calculation on there
        // would be good to plot response etc

        // filter: Filter::new(),
        // filter: Filter::lowpass(256, 44100.0, 400.0),
        mixer: Mixer::new(sample_rate),

        channel,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            on_window(output, &mut request, on_sample)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F)
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static,
{
    if let Some(msg) = request.channel.pop() {
        match msg {
            SoundMessage::PlaySound(s, id) => {
                request.mixer.add_sound(s, id);
            },
            SoundMessage::StopSound(id) => {
                request.mixer.stop_sound(id);
            }
        }
    }
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
