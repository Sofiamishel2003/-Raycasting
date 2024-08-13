mod maze;
mod framebuffer;
mod player;
mod sounds; 

use minifb::{Key, Window, WindowOptions};
use core::{f32::consts::PI, num};
use nalgebra_glm::Vec2;
use player::{Player, process_events};
use std::{os::windows::io::FromRawHandle, process::Termination, time::Duration};
use framebuffer::Framebuffer;
use maze::load_maze;
use rusttype::{Font, Scale};
use image::{RgbaImage, Rgba};
use sounds::{play_background_music, play_victory_sound}; // Importar funciones de sonido

use once_cell::sync::Lazy;
use std::sync::Arc; 

mod ray_casting;
use ray_casting::{cast_ray, cast_ray_minimap, Intersect};

mod texture;
use texture::Texture;

static PARED: Lazy<Arc<Texture>> = Lazy::new(||  Arc::new(Texture::new("src\\assets\\images\\green_texture.jpg")));
static PUERTA: Lazy<Arc<Texture>> = Lazy::new(||  Arc::new(Texture::new("src\\assets\\images\\door.png")));


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
        '+'  | '|'| '-' => 0x345f01 ,  
        'g' => 0xFF0000,                                
        _ => 0x9fbf7a ,                                  
    };

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
        }
    }
}

fn render2d(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>){
    let block_size = 100; 

    for row in 0..maze.len(){
        for col in 0..maze[row].len(){
            draw_cell(framebuffer, col * block_size, row * block_size, block_size, maze[row][col]);
        }
    }

    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.point(player.pos.x as usize, player.pos.y as usize); 

    let num_rays = 100; 

    for i in 0..num_rays {
        let current_ray = i as f32/ num_rays as f32; 
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray); 
        cast_ray(framebuffer, &maze, player, a, block_size, true); 
    }
}

fn render3d(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>){

    let block_size = 100; 

    let hh =  framebuffer.height as f32/2.0;
    
    let num_rays = framebuffer.width; 

    
    // Establecer el color para el cielo (parte superior) y el suelo (parte inferior)
    let sky_color = 0x87CEEB; // Celeste
    let ground_color = 0x006400; // Verde oscuro
    
    for i in 0..framebuffer.width {
        framebuffer.set_current_color(sky_color);
        for j in 0..(framebuffer.height / 2){
            framebuffer.point(i, j);
        }
    
        framebuffer.set_current_color(ground_color);
        for j in (framebuffer.height / 2)..framebuffer.height{
            framebuffer.point(i, j);
        }
    }

    for i in 0..num_rays {
        let current_ray = i as f32/ num_rays as f32; 
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray); 
        let intersect = cast_ray(framebuffer, &maze, player, a, block_size, false);


        let distance = intersect.distance * (a - player.a).cos();

        let stake_height = (framebuffer.height as f32 / distance) * 50.0; 

        let stake_top = (hh - (stake_height / 2.0 )) as usize; 
        let stake_bottom = (hh + (stake_height / 2.0 )) as usize;

        for y in stake_top..stake_bottom{
            let ty = (y as f32- stake_top as f32) / (stake_bottom  as f32 - stake_top as f32) * 128.0;
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


fn main() {
    let window_width = 1300;
    let window_height = 900;

    let framebuffer_width = 1300;
    let framebuffer_height = 900;

    let frame_delay = Duration::from_millis(0);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "BRAT MAZE",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(100, 100);
    window.update();

    // initialize values
    framebuffer.set_background_color(0x333355);
    play_background_music("src/assets/music/Background_Music.mp3");

    window.set_cursor_visibility(false); 

    let maze = load_maze("./mazes/maze1.txt");

    let mut player = Player{
        pos: Vec2::new(150.0, 150.0),
        a: PI/1.8, 
        fov: PI/4.0,
    };

    let mut mode = "3D"; 

    let minimap_scale = 0.2;
    let minimap_width = (framebuffer.width as f32 * minimap_scale) as usize;
    let minimap_height = (framebuffer.height as f32 * minimap_scale) as usize;
    let minimap_x = framebuffer.width - minimap_width - 20;
    let minimap_y = framebuffer.height - minimap_height - 20;

    while window.is_open(){
        //listen to inputs
        if window.is_key_down(Key::Escape){
            break;
        }
    
        if window.is_key_down(Key::M){
            mode = if mode == "2D" {"3D"} else {"2D"}; 
        }

        process_events(&window, &mut player, &maze);

        framebuffer.clear();
    
        if mode == "2D"{
            render2d(&mut framebuffer, &player, &maze);
        } else {
            render3d(&mut framebuffer, &player, &maze)
        }

        render_minimap(&mut framebuffer, &player, &maze, minimap_x, minimap_y, minimap_scale);

        // Actualizar la ventana con el buffer del framebuffer
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();

        std::thread::sleep(frame_delay);
    }
}

