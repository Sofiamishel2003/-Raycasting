mod framebuffer;
mod maze;

use framebuffer::Framebuffer;
use maze::load_maze;
use minifb::{Key, Window, WindowOptions};

fn draw_cell(framebuffer: &mut Framebuffer, x0: usize, y0: usize, block_size: usize, cell: char) {
    match cell {
        '+' | '-' => framebuffer.set_current_color(0x333355), // Color de las paredes
        'p' => framebuffer.set_current_color(0x00FF00),        // Color del punto de inicio
        'g' => framebuffer.set_current_color(0xFF0000),        // Color del objetivo
        ' ' => framebuffer.set_current_color(0xFFDAB9),        // Color del espacio vacío
        _ => framebuffer.set_current_color(0x000000),          // Color por defecto (negro)
    }

    // Dibuja un rectángulo en la posición (x0, y0) con tamaño block_size
    for y in y0..y0 + block_size {
        for x in x0..x0 + block_size {
            framebuffer.point(x as isize, y as isize);
        }
    }
}

fn render(framebuffer: &mut Framebuffer) {
    let maze = load_maze("./maze.txt");
    let block_size = 100; // Tamaño del bloque aumentado

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            draw_cell(
                framebuffer,
                col * block_size,
                row * block_size,
                block_size,
                maze[row][col],
            );
        }
    }
}

fn main() {
    let maze = load_maze("./maze.txt");
    let block_size = 100; // Tamaño del bloque aumentado

    let window_width = maze[0].len() * block_size;  // Basado en el tamaño del laberinto
    let window_height = maze.len() * block_size;    // Basado en el tamaño del laberinto

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    let mut window = Window::new(
        "Maze Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Render the maze to the framebuffer
        render(&mut framebuffer);

        // Update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer(), window_width, window_height)
            .unwrap();
    }
}
