use std::fs::File;
use std::io::Write;
use std::thread;

type Vec2 = (f32, f32);
type Vec4 = (f32, f32, f32, f32);

const SIZE: usize = 600;
const NUM_FRAMES: usize = 60;
const FPS: f32 = 30.0;
const DURATION: f32 = 2.0 * std::f32::consts::PI;
const SCALE: f32 = 0.65;

fn main() {
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(NUM_FRAMES);

    std::fs::create_dir_all("frames").expect("无法创建 frames 目录");
    println!(
        "使用 {} 个线程生成 {} 帧动画（{}x{} 方形）...",
        num_threads, NUM_FRAMES, SIZE, SIZE
    );

    let frames_per_thread = (NUM_FRAMES as f32 / num_threads as f32).ceil() as usize;
    let handles: Vec<_> = (0..num_threads)
        .filter_map(|thread_id| {
            let start = thread_id * frames_per_thread;
            let end = (start + frames_per_thread).min(NUM_FRAMES);
            (start < NUM_FRAMES)
                .then(|| thread::spawn(move || generate_frames(thread_id, start..end)))
        })
        .collect();

    handles
        .into_iter()
        .for_each(|h| h.join().expect("线程执行失败"));

    println!("所有帧已保存到 frames/ 目录");
    println!("使用以下命令转换为 GIF：");
    println!(
        "  ffmpeg -framerate {} -i frames/frame_%04d.ppm -y output.gif",
        FPS as u32
    );
}

fn generate_frames(thread_id: usize, frames: std::ops::Range<usize>) {
    for frame in frames {
        let time = if NUM_FRAMES > 1 {
            (frame as f32 / (NUM_FRAMES - 1) as f32) * DURATION
        } else {
            0.0
        };

        if frame % 10 == 0 {
            println!(
                "线程 {}: 生成第 {}/{} 帧...",
                thread_id,
                frame + 1,
                NUM_FRAMES
            );
        }

        let mut pixels = vec![0u8; SIZE * SIZE * 3];
        for y in 0..SIZE {
            for x in 0..SIZE {
                let (r, g, b, _) = compute_pixel(
                    (x as f32 + 0.5, y as f32 + 0.5),
                    (SIZE as f32, SIZE as f32),
                    time,
                );
                let idx = (y * SIZE + x) * 3;
                pixels[idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }

        write_ppm_binary(&format!("frames/frame_{:04}.ppm", frame), &pixels);
    }
}

fn compute_pixel(fc: Vec2, r: Vec2, t: f32) -> Vec4 {
    let scale = r.1 * SCALE;
    let p = ((fc.0 * 2.0 - r.0) / scale, (fc.1 * 2.0 - r.1) / scale);
    let l = (0.7 - (p.0 * p.0 + p.1 * p.1)).abs();
    let mut v = (p.0 * (1.0 - l) / 0.2, p.1 * (1.0 - l) / 0.2);
    let mut o = (0.0, 0.0, 0.0, 0.0);

    for i in 1..=8 {
        let i_f = i as f32;
        let sin_v = (v.0.sin(), v.1.sin(), v.1.sin(), v.0.sin());
        let abs_diff = (v.0 - v.1).abs();
        o = (
            o.0 + (sin_v.0 + 1.0) * abs_diff * 0.2,
            o.1 + (sin_v.1 + 1.0) * abs_diff * 0.2,
            o.2 + (sin_v.2 + 1.0) * abs_diff * 0.2,
            o.3 + (sin_v.3 + 1.0) * abs_diff * 0.2,
        );
        v = (
            v.0 + (v.1 * i_f + t).cos() / i_f + 0.7,
            v.1 + (v.0 * i_f + i_f + t).cos() / i_f + 0.7,
        );
    }

    let exp_py = (p.1.exp(), (-p.1).exp(), (-p.1 * 2.0).exp(), 1.0);
    let exp_l = (-4.0 * l).exp();
    let o_safe = (
        o.0.abs().max(1e-10),
        o.1.abs().max(1e-10),
        o.2.abs().max(1e-10),
        o.3.abs().max(1e-10),
    );

    (
        (exp_py.0 * exp_l / o_safe.0).tanh(),
        (exp_py.1 * exp_l / o_safe.1).tanh(),
        (exp_py.2 * exp_l / o_safe.2).tanh(),
        (exp_py.3 * exp_l / o_safe.3).tanh(),
    )
}

fn write_ppm_binary(filename: &str, pixels: &[u8]) {
    let mut file = File::create(filename).expect("无法创建文件");
    writeln!(file, "P6\n{} {}\n255", SIZE, SIZE).expect("写入失败");
    file.write_all(pixels).expect("写入失败");
}
