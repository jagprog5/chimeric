use std::{num::NonZero, path::Path, thread::sleep, time::Duration};

use chimeric_engine::core::system::{ChimericSystem, ChimericSystemSettings, System};
use sdl2::rect::Rect;

fn main() -> std::process::ExitCode {
    let system = System::new().unwrap();
    let mut chimeric_system = ChimericSystem::new(&system, ChimericSystemSettings {
        num_point_sizes_per_font: NonZero::new(100).unwrap(),
        num_fonts: NonZero::new(5).unwrap(),
        num_textures_per_window: NonZero::new(100).unwrap(),
    });
    let window = system.video
        .window("shift tab! mouse!", 200, 200)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    chimeric_system.add_window("main", window).unwrap();

    let image_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("test.jpg");

    let font_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("TEMPSITC-REDUCED.TTF");
    
    chimeric_system.copy("main", &image_path, None, None).unwrap();
    chimeric_system.copy_text("main", &font_path, 50, c"text", None, None, Rect::new(0, 0, 200, 50)).unwrap();
    chimeric_system.present();

    sleep(Duration::from_secs(2));

    std::process::ExitCode::SUCCESS
}
