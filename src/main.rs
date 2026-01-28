

/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

/*
So i guess the main thing is putting in compelling synth options and then gui controls for them
its easy enough
fun to design
like maybe instead of enum for the oscs just have magnitude for each one

from a pure code perspective just f(sample number) could do anything. like cooked fm synthesis, how to make jump up sounds
yeah fm is juicy

also stuff like filter envelope

oh sheesh can do stuff like additive, draw a spectrogram
shepherd tones
look into PSGs in snes etc

wonder what feedback can do
acid bass: square wave + pluck adsr on filter + high reso on filter
could make a game with procedural music that is impacted by your actions etc

bezier curve muzik
patterns

rhythm game
maybe tap up or down to change lanes, and music goes with it
if you need to change 2 lanes, a double tap its swung 
make it super fast, like tapping out an insane jungle drum pattern. having to feel hectic rhythms in the music, can you
sound slike super flow state

different powerups change music, affect scoring, multiplier etc.
eg dub zone applies echo. maybe collect combinationss
*/

extern crate anyhow;
extern crate clap;
extern crate cpal;

use application::*;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod kmath;
mod kimg;
mod kinput;
mod krenderer;
mod application;
mod synth;
mod filter;
mod sound;
mod keyboard;
mod fftviewer;
mod envelope;
use crate::kmath::*;
use crate::synth::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() -> anyhow::Result<()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let mut application = Application::new(&event_loop);
    
    event_loop.run(move |event, _, control_flow| {
        application.handle_event(&event);
        match event {
            Event::LoopDestroyed |
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..}
            => {
                *control_flow = ControlFlow::Exit;
            },
            _ => (),
        }
    });
    Ok(())
}
