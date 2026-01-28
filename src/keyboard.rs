use crate::krenderer::*;
use crate::kinput::*;
use crate::kmath::*;
use glutin::event::VirtualKeyCode;

const keys: [VirtualKeyCode; 23] = [
    VirtualKeyCode::Tab,    // A
    VirtualKeyCode::Key1,   // Bb
    VirtualKeyCode::Q,      // B
    VirtualKeyCode::W,      // C
    VirtualKeyCode::Key3,   // Db
    VirtualKeyCode::E,      // D
    VirtualKeyCode::Key4,   // Eb
    VirtualKeyCode::R,      // E
    VirtualKeyCode::T,      // F
    VirtualKeyCode::Key6,   // Gb
    VirtualKeyCode::Y,      // G
    VirtualKeyCode::Key7,   // Ab
    VirtualKeyCode::U,      // A
    VirtualKeyCode::Key8,   // Bb
    VirtualKeyCode::I,      // B
    VirtualKeyCode::O,      // C
    VirtualKeyCode::Key0,   // Db
    VirtualKeyCode::P,      // D
    VirtualKeyCode::Minus,       // Eb
    VirtualKeyCode::LBracket,   // E
    VirtualKeyCode::RBracket,   // F
    VirtualKeyCode::Back,  // Gb
    VirtualKeyCode::Backslash,  // G
];
const white_keys: [VirtualKeyCode; 14] = [
    VirtualKeyCode::Tab,    // A
    VirtualKeyCode::Q,      // B
    VirtualKeyCode::W,      // C
    VirtualKeyCode::E,      // D
    VirtualKeyCode::R,      // E
    VirtualKeyCode::T,      // F
    VirtualKeyCode::Y,      // G
    VirtualKeyCode::U,      // A
    VirtualKeyCode::I,      // B
    VirtualKeyCode::O,      // C
    VirtualKeyCode::P,      // D
    VirtualKeyCode::LBracket,   // E
    VirtualKeyCode::RBracket,   // F
    VirtualKeyCode::Backslash,  // G
];
const black_keys: [VirtualKeyCode; 9] = [
    VirtualKeyCode::Key1,   // Bb
    VirtualKeyCode::Key3,   // Db
    VirtualKeyCode::Key4,   // Eb
    VirtualKeyCode::Key6,   // Gb
    VirtualKeyCode::Key7,   // Ab
    VirtualKeyCode::Key8,   // Bb
    VirtualKeyCode::Key0,   // Db
    VirtualKeyCode::Minus,       // Eb
    VirtualKeyCode::Back,       // Gb
];

pub struct Keyboard {
    current_octave: i32,
    held_keys: Vec<u32>,
    counters: Vec<u32>,
}

#[derive(Debug)]
pub struct KeyboardEvent {
    pub uid: u32,
    pub freq: f32,
    pub pressed: bool, // else released
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            current_octave: 0,
            held_keys: Vec::new(),
            counters: vec![0; keys.len()],
        }
    }

    // lets plot fft

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas, rect: Rect) -> Vec<KeyboardEvent> {
        let mut events = Vec::new();

        // if they switch octave unhold those keys hey
        if inputs.key_rising(VirtualKeyCode::LShift) {
            self.current_octave += 1;
        }
        if inputs.key_rising(VirtualKeyCode::LControl) {
            self.current_octave -= 1;
        }

        let base_freq = 440. * 2.0f32.powf(self.current_octave as f32);
        for i in 0..keys.len() {
            if inputs.key_rising(keys[i as usize]) {
                self.held_keys.push(i as u32);
                self.counters[i as usize] += 1;
                events.push(KeyboardEvent { 
                    uid: khash(self.counters[i as usize]) * khash(i as u32),
                    freq: base_freq * 2.0f32.powf(i as f32/12.0),
                    pressed: true,
                });
            }
        }

        // next up integral sliders! coming together
        // then maybe fmod for the waves lol.
        // then maybe attempt sallen key filters, that will be mad...
        // also can probably clean up and put on github before bad things happen

        // can you convolve the frequency response with something to shift it
        


        // kill everything that the keys not held for
        // might not need to do the octave resettings
        // layout: white keys are easy
        // what if we assumed all whtie and all black and then had false

        let released: Vec<u32> = self.held_keys
            .iter()
            .filter(|k| !inputs.key_held(keys[**k as usize]))
            .copied()
            .collect();
        for k in &released {
            events.push(KeyboardEvent {
                uid: khash(self.counters[*k as usize]) * khash(*k),
                freq: 0.0,
                pressed: false,
            });
        }
        self.held_keys.retain(|k| inputs.key_held(keys[*k as usize]));

        kc.set_depth(1.5);
        kc.set_colour(Vec4::new(0.0, 0.0, 0.0, 1.0));
        kc.rect(rect);
        kc.set_depth(1.6);
        let spaces = rect.split_lrn(white_keys.len() as i32 + 1);

        // Octave up&down buttons
        let (oct_up, oct_down) = spaces[0].split_ud(0.5);
        kc.set_colour(if inputs.key_held(VirtualKeyCode::LShift) {
            Vec4::new(1.0, 0.9, 0.9, 1.0)
        } else {
            Vec4::new(1.0, 0.5, 0.5, 1.0)
        });
        kc.rect(oct_up.dilate_pc(-0.05));
        kc.set_colour(if inputs.key_held(VirtualKeyCode::LControl) {
            Vec4::new(0.9, 0.9, 1.0, 1.0)
        } else {
            Vec4::new(0.5, 0.5, 1.0, 1.0)
        });
        kc.rect(oct_down.dilate_pc(-0.05));

        // White keys
        for i in 1..spaces.len() {
            kc.set_colour(if inputs.key_held(white_keys[i-1 as usize]) {
                Vec4::new(0.8, 0.8, 0.8, 1.0)
            } else {
                Vec4::new(1.0, 1.0, 1.0, 1.0)
            });
            kc.rect(spaces[i as usize].dilate_pc(-0.05));
        }

        // black keys have fun
        kc.set_depth(1.7);
        let br = |r: Rect| Rect::new(r.x + r.w*0.75, r.y, r.w * 0.5, r.h * 0.5);
        for (i, bki) in [0, 2, 3, 5, 6, 7, 9, 10, 12].iter().enumerate() {
            let bk_rect = br(spaces[(1 + bki) as usize]);
            kc.set_colour(if inputs.key_held(black_keys[i]) {
                Vec4::new(0.2, 0.2, 0.2, 1.0)
            } else {
                Vec4::new(0.0, 0.0, 0.0, 1.0)
            });
            kc.rect(bk_rect);
        }

        events
    }
}