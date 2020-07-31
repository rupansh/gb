use crate::consts::*;
use crate::{
    cpu::Cpu,
    mem::Mem,
    input::{KeyCode, Input},
};
use sdl2::{
    event::Event,
    EventPump,
    keyboard::Keycode,
    rect::Rect,
    render::Canvas,
};

pub struct FrontEnd {
    canvas: Canvas<sdl2::video::Window>,
    event_pump: EventPump
}

impl Default for FrontEnd {
    fn default() -> FrontEnd {
        let ctx = sdl2::init().unwrap();
        let mut front = FrontEnd { 
            canvas: ctx.video().unwrap()
                .window("Gameboy Emu", (WIDTH*SCALE) as u32, (HEIGHT*SCALE) as u32)
                .position_centered()
                .build().unwrap()
                .into_canvas()
                .build().unwrap(),
            event_pump: ctx.event_pump().unwrap(),
        };

        front.canvas.set_draw_color(PAL_0);
        front.canvas.clear();
        front.canvas.present();

        return front;
    }
}

impl FrontEnd {
    pub fn draw_pix(&mut self, x: i32, y: i32, col: (u8, u8, u8)) {
        self.canvas.set_draw_color(col);

        let r = Rect::new(x*SCALE as i32, y*SCALE as i32, SCALE as u32, SCALE as u32);

        self.canvas.fill_rect(r).unwrap_or_default();
    }

    pub fn render(&mut self) {
        self.canvas.present();
    }

    fn input_key(key: Keycode) -> KeyCode {
        match key {
            Keycode::Return => KeyCode::Start,
            Keycode::Space => KeyCode::Select,
            Keycode::E => KeyCode::B,
            Keycode::Q => KeyCode::A,
            Keycode::S => KeyCode::Down,
            Keycode::W => KeyCode::Up,
            Keycode::A => KeyCode::Left,
            Keycode::D => KeyCode::Right,
            _ => KeyCode::Uk
        }
    }

    pub fn check_event(&mut self, cpu: &mut Cpu, input: &mut Input, mem: &mut Mem) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    cpu.stop = 1;
                },
                Event::KeyDown { keycode: Some(k), .. } => { input.key(mem, FrontEnd::input_key(k), true) },
                Event::KeyUp { keycode: Some(k), .. } => input.key(mem, FrontEnd::input_key(k), false),
                _ => {}
            }
        }
    }
}