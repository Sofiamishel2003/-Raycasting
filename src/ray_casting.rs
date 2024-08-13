
use crate::framebuffer::Framebuffer;
use crate::player::Player; 

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub tx: usize,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
    player: &Player,
    a: f32, 
    block_size: usize,
    draw_line: bool,
    
) -> Intersect {
    let mut d = 0.0; 


    framebuffer.set_current_color(0xFFFFFF);
    loop {

        let cos = d * a.cos();
        let sin = d * a.sin();

        let x = (player.pos.x + cos) as usize; 
        let y = (player.pos.y + sin) as usize;

        let i = x / block_size;
        let j = y / block_size; 

        let hitx = x - i * block_size;
        let hity = y - j * block_size;

        let mut maxhit = hity; 

        if 1 < hitx && hitx < block_size - 1 {
            maxhit = hitx;
        }

        if draw_line {
            framebuffer.point(x,y);
        }

        if maze[j][i] != ' ' {
            return Intersect {
                distance: d, 
                impact: maze[j][i],
                tx: maxhit * 128 / block_size, 
            };
        }
        d += 1.0; 
    }
}

pub fn cast_ray_minimap(framebuffer: &mut Framebuffer, maze: &Vec<Vec<char>>, player: &Player, angle: f32, block_size: usize, minimap_x: usize, minimap_y: usize, scale: f32) {
    let mut d = 0.0;
    let max_distance = 50.0; 

    while d < max_distance {
        let cos = d * angle.cos();
        let sin = d * angle.sin();

        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;

        // Ajusta las coordenadas para el minimapa
        let mx = minimap_x + (x as f32 * scale) as usize;
        let my = minimap_y + (y as f32 * scale) as usize;

        if mx >= framebuffer.width || my >= framebuffer.height {
            break; // Evita dibujar fuera de los límites del framebuffer
        }

        framebuffer.set_current_color(0xFFFFFF);
        framebuffer.point(mx, my);

        let i = x / block_size;
        let j = y / block_size;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'#') { // '#' representa un muro
            break;
        }
        d += 1.0;
    }
}

