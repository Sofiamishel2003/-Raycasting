mod maze;
mod framebuffer;
mod player;
mod sounds;

use minifb::{Key, Window, WindowOptions};
use core::{f32::consts::PI};
use nalgebra_glm::Vec2;
use player::{Player, process_events};
use std::{time::{Duration, Instant}};
use framebuffer::Framebuffer;
use maze::load_maze;
use sounds::{play_background_music, play_victory_sound, stop_music, play_screamer_sound};

use once_cell::sync::Lazy;
use std::sync::Arc;
use image::RgbaImage;

mod ray_casting;
use ray_casting::{cast_ray, cast_ray_minimap};

mod texture;
use texture::Texture;

static PARED: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("src\\assets\\images\\green_texture.jpg")));
static PUERTA: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("src\\assets\\images\\door.png")));
static SCREAMER_IMAGE: Lazy<RgbaImage> = Lazy::new(|| image::open("src\\assets\\images\\screamer.png").unwrap().to_rgba8());

fn draw_text(framebuffer: &mut Framebuffer, text: &str, x: usize, y: usize, color: u32) {
    let font = vec![
        // Define a simple 8x8 font bitmap for characters 0-9 (ASCII 48-57)
        0x3E, 0x51, 0x49, 0x45, 0x3E, // 0
        0x00, 0x42, 0x7F, 0x40, 0x00, // 1
        0x42, 0x61, 0x51, 0x49, 0x46, // 2
        0x21, 0x41, 0x45, 0x4B, 0x31, // 3
        0x18, 0x14, 0x12, 0x7F, 0x10, // 4
        0x27, 0x45, 0x45, 0x45, 0x39, // 5
        0x3C, 0x4A, 0x49, 0x49, 0x30, // 6
        0x01, 0x71, 0x09, 0x05, 0x03, // 7
        0x36, 0x49, 0x49, 0x49, 0x36, // 8
        0x06, 0x49, 0x49, 0x29, 0x1E, // 9
    ];

    let char_width = 5; // Ancho de cada carácter
    let char_height = 8; // Alto de cada carácter

    for (i, c) in text.chars().enumerate() {
        if c >= '0' && c <= '9' {
            let offset = (c as usize - '0' as usize) * char_width;
            for row in 0..char_height {
                for col in 0..char_width {
                    if font[offset + col] & (1 << (char_height - 1 - row)) != 0 {
                        framebuffer.set_current_color(color);
                        framebuffer.point(x + col + i * (char_width + 1), y + row);
                    }
                }
            }
        }
    }
}

fn draw_image(framebuffer: &mut Framebuffer, image: &RgbaImage, x: usize, y: usize, scale: f32) {
    let scaled_width = (image.width() as f32 * scale) as usize;
    let scaled_height = (image.height() as f32 * scale) as usize;

    for (i, pixel) in image.pixels().enumerate() {
        let px = i % image.width() as usize;
        let py = i / image.width() as usize;

        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;
        let a = pixel[3] as f32 / 255.0;

        if a == 0.0 {
            continue;
        }

        let current_color = framebuffer.get_pixel_color(x + px, y + py);

        let current_r = ((current_color >> 16) & 0xFF) as f32;
        let current_g = ((current_color >> 8) & 0xFF) as f32;
        let current_b = (current_color & 0xFF) as f32;

        let blended_r = (r * a + current_r * (1.0 - a)) as u32;
        let blended_g = (g * a + current_g * (1.0 - a)) as u32;
        let blended_b = (b * a + current_b * (1.0 - a)) as u32;

        let blended_color = (blended_r << 16) | (blended_g << 8) | blended_b;

        for sx in 0..(scaled_width / image.width() as usize) {
            for sy in 0..(scaled_height / image.height() as usize) {
                framebuffer.set_current_color(blended_color);
                framebuffer.point(x + px * (scaled_width / image.width() as usize) + sx, y + py * (scaled_height / image.height() as usize) + sy);
            }
        }
    }
}

fn cell_to_texture_color(cell: char, tx: u32, ty: u32) -> u32 {
    let default_color = 0x000000;

    match cell {
        '+' => PARED.get_pixel_color(tx, ty),
        '-' => PARED.get_pixel_color(tx, ty),
        '|' => PARED.get_pixel_color(tx, ty),
        'g' => PUERTA.get_pixel_color(tx, ty),
        _ => default_color,
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    let color = match cell {
        '+' | '|' | '-' => 0x345f01,
        'g' => 0xFF0000,
        _ => 0x9fbf7a,
    };

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
        }
    }
}

fn render2d(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>) {
    let block_size = 100;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            draw_cell(framebuffer, col * block_size, row * block_size, block_size, maze[row][col]);
        }
    }

    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.point(player.pos.x as usize, player.pos.y as usize);

    let num_rays = 100;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, player, a, block_size, true);
    }
}

fn render3d(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>) {
    let block_size = 100;
    let hh = framebuffer.height as f32 / 2.0;
    let num_rays = framebuffer.width;

    let sky_color = 0x87CEEB;
    let ground_color = 0x006400;

    for i in 0..framebuffer.width {
        framebuffer.set_current_color(sky_color);
        for j in 0..(framebuffer.height / 2) {
            framebuffer.point(i, j);
        }

        framebuffer.set_current_color(ground_color);
        for j in (framebuffer.height / 2)..framebuffer.height {
            framebuffer.point(i, j);
        }
    }

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, player, a, block_size, false);

        let distance = intersect.distance * (a - player.a).cos();
        let stake_height = (framebuffer.height as f32 / distance) * 50.0;

        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        for y in stake_top..stake_bottom {
            let ty = (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32) * 128.0;
            let tx = intersect.tx;

            let color = cell_to_texture_color(intersect.impact, tx as u32, ty as u32);
            framebuffer.set_current_color(color);
            framebuffer.point(i, y);
        }
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>, minimap_x: usize, minimap_y: usize, minimap_scale: f32) {
    let block_size = (100.0 * minimap_scale) as usize;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            let cell = maze[row][col];
            let xo = minimap_x + col * block_size;
            let yo = minimap_y + row * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }

    let player_x = minimap_x + (player.pos.x as f32 * minimap_scale) as usize;
    let player_y = minimap_y + (player.pos.y as f32 * minimap_scale) as usize;
    framebuffer.set_current_color(0xFF0000);
    framebuffer.point(player_x, player_y);

    let num_rays = 50;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let angle = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray_minimap(framebuffer, &maze, player, angle, block_size, minimap_x, minimap_y, minimap_scale);
    }
}

fn calculate_fps(last_frame_time: &mut Instant) -> u32 {
    let duration = last_frame_time.elapsed();
    let fps = 1.0 / duration.as_secs_f32();
    *last_frame_time = Instant::now();
    fps as u32
}

fn main() {
    let window_width = 1300;
    let window_height = 900;

    let framebuffer_width = 1300;
    let framebuffer_height = 900;

    let frame_delay = Duration::from_millis(0);
    let mut last_frame_time = Instant::now(); // Definir last_frame_time

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    // Mostrar pantalla inicial y selección de laberinto
    let mut window = Window::new(
        "BRAT MAZE - Selección de Laberinto",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    play_background_music("src/assets/music/Menu_Music.mp3");

    let menu_image = image::open("src/assets/images/menu_image.png").unwrap().to_rgba8();

    let mut selected_maze = String::new();

    while window.is_open() {
        let fps = calculate_fps(&mut last_frame_time);  // Calcular FPS
        let fps_text = format!("FPS: {}", fps);
        let fps_x = framebuffer_width - (fps_text.len() * 8) - 10;  // Calcula la posición x para alinear el texto a la derecha
        
        if window.is_key_down(Key::Key1) {
            selected_maze = "./mazes/maze1.txt".to_string();
            stop_music(); // Detener la música del menú
            play_background_music("src/assets/music/Club_classics.mp3");
            break;
        } else if window.is_key_down(Key::Key2) {
            selected_maze = "./mazes/maze2.txt".to_string();
            stop_music(); // Detener la música del menú
            play_background_music("src/assets/music/360.mp3");
            break;
        } else if window.is_key_down(Key::Key3) {
            selected_maze = "./mazes/maze3.txt".to_string();
            stop_music(); // Detener la música del menú
            play_background_music("src/assets/music/Girl_so_confusing.mp3");
            break;
        }

        framebuffer.clear();
        draw_image(&mut framebuffer, &menu_image, 0, 0, 1.0); // Mostrar imagen del menú

        draw_text(&mut framebuffer, &fps_text, fps_x, 10, 0xFFFFFF);  // Mostrar FPS en la esquina superior derecha
        
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }

    drop(window);

    // Cargar laberinto seleccionado
    let maze = load_maze(&selected_maze);

    // Encontrar la posición de la meta
    let mut goal_row = 0;
    let mut goal_col = 0;

    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            if cell == 'g' {
                goal_row = row;
                goal_col = col;
                break;
            }
        }
    }

    let mut player = Player {
        pos: Vec2::new(150.0, 150.0),
        a: PI / 1.8,
        fov: PI / 4.0,
    };

    let mut window = Window::new(
        "BRAT MAZE",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_cursor_visibility(false);

    let mut mode = "3D";
    let mut victory_achieved = false;
    let mut screamer_triggered = false;
    let mut screamer_scale = 0.1;
    let mut last_screamer_time = Instant::now();

    let minimap_scale = 0.2;
    let minimap_width = (framebuffer.width as f32 * minimap_scale) as usize;
    let minimap_height = (framebuffer.height as f32 * minimap_scale) as usize;
    let minimap_x = framebuffer.width - minimap_width - 20;
    let minimap_y = framebuffer.height - minimap_height - 20;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
    
        if window.is_key_down(Key::M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }
    
        process_events(&window, &mut player, &maze);
    
        framebuffer.clear();
    
        if mode == "2D" {
            render2d(&mut framebuffer, &player, &maze);
        } else {
            render3d(&mut framebuffer, &player, &maze);
        }
    
        render_minimap(&mut framebuffer, &player, &maze, minimap_x, minimap_y, minimap_scale);
    
        // Verificar si el screamer debe activarse cada 5 segundos
        if last_screamer_time.elapsed().as_secs() >= 10 {
            screamer_triggered = true;
            last_screamer_time = Instant::now(); // Resetear el temporizador
            play_screamer_sound(); // Reproducir sonido del screamer
        }

        // Dibujar el screamer si se activó
        if screamer_triggered {
            draw_image(&mut framebuffer, &SCREAMER_IMAGE, 300, 200, screamer_scale);
            screamer_scale += 0.05; // Aumentar el tamaño del screamer para la animación
            if screamer_scale >= 1.5 {
                screamer_triggered = false; // Ocultar el screamer después de un tiempo
                screamer_scale = 0.1; // Reiniciar el tamaño del screamer
            }
        }
    
        let fps = calculate_fps(&mut last_frame_time);  // Calcular FPS
        draw_text(&mut framebuffer, &format!("FPS: {}", fps), 10, 10, 0xFFFFFF);  // Mostrar FPS en la esquina superior izquierda

        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
    
        // Verificar si el jugador alcanzó la meta (g) o está en una celda adyacente
        let player_col = (player.pos.x as usize) / 100;
        let player_row = (player.pos.y as usize) / 100;

        let row_diff = (goal_row as isize - player_row as isize).abs();
        let col_diff = (goal_col as isize - player_col as isize).abs();
    
        // Si el jugador está en la meta o en una celda adyacente (diagonal incluida)
        if row_diff <= 1 && col_diff <= 1 {
            victory_achieved = true;
            break;
        }
    
        std::thread::sleep(frame_delay);
    }
    
    // Solo mostrar la pantalla de victoria si el jugador ha ganado
    if victory_achieved {
        stop_music(); // Detener la música del juego
        play_victory_sound();

        let victory_image = image::open("src/assets/images/victory_image.png").unwrap().to_rgba8();

        let mut window = Window::new(
            "FELICIDADES",
            window_width,
            window_height,
            WindowOptions::default(),
        ).unwrap();

        framebuffer.clear();  // Asegurarse de que el framebuffer está limpio
        draw_image(&mut framebuffer, &victory_image, 0, 0, 1.0);  // Mostrar la imagen de felicitaciones

        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
            std::thread::sleep(frame_delay);
        }
    }
}
