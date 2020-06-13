extern crate sdl2;
mod consts;
mod cpu;
mod input;
mod gpu;
mod mem;
mod timer;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn load_rom(gb_mem: &mut mem::Mem, rom: &str) -> io::Result<()> {
    let mut r = File::open(rom)?;
    r.read_exact(&mut gb_mem.rom)?;
    r.seek(SeekFrom::Start(16384))?;
    r.read_exact(&mut gb_mem.rom_bank)?;
    return Ok(())
}

fn main() -> io::Result<()> {
    let mut gb_cpu = cpu::Cpu::default();
    let mut gb_gpu = gpu::Gpu::default();
    let mut gb_mem = mem::Mem::default();
    let mut gb_input = input::Input::default();
    let mut gb_timer = timer::Timer::default();
    load_rom(&mut gb_mem, "test_roms/instr_timing.gb")?;
    gb_mem.io[0] = 255;
    gb_mem.io[41] = 0x80;
    gb_mem.write(consts::CTLTTP, 3);
    gb_mem.write(consts::JOYP, 255);
    gb_exec(&mut gb_cpu, &mut gb_gpu, &mut gb_input, &mut gb_timer, &mut gb_mem).unwrap();
    return Ok(())

}

fn gb_frame(gb_cpu: &mut cpu::Cpu, gb_gpu: &mut gpu::Gpu, gb_input: &mut input::Input, gb_timer: &mut timer::Timer, gb_mem: &mut mem::Mem) {
    let target = gb_cpu.clk.val + 70224;
    while gb_cpu.clk.val < target && gb_cpu.stop == 0 {
        cpu::cpu_cycle(gb_cpu, gb_mem);
        gpu::gpu_cycle(gb_gpu, gb_mem, &gb_cpu.clk);
        if gb_mem.input_update {
            gb_input.update(gb_mem);
        }
        gb_timer.inc(&gb_cpu.clk, gb_mem);
    }
}

fn gb_exec(gb_cpu: &mut cpu::Cpu, gb_gpu: &mut gpu::Gpu, gb_input: &mut input::Input, gb_timer: &mut timer::Timer, gb_mem: &mut mem::Mem) -> Result<(), String> {
    let mut event_pump = gb_gpu.ctx.event_pump()?;
    let st = std::time::Instant::now();
    let mut frame = 0.;
    while gb_cpu.stop == 0 {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    gb_cpu.stop = 1;
                },
                Event::KeyDown { keycode: Some(k), .. } => { gb_input.key(gb_mem, k, true) },
                Event::KeyUp { keycode: Some(k), .. } => gb_input.key(gb_mem, k, false),
                _ => {}
            }
        }
        gb_frame(gb_cpu, gb_gpu, gb_input, gb_timer, gb_mem);
        //gb_gpu.canvas.present();
        frame += 1.;
        //std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let ep = st.elapsed();
    println!("{}", frame/ep.as_secs_f64());
    Ok(())
}