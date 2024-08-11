pub struct Framebuffer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Framebuffer {
        let buffer = vec![0xFFFFFF; width * height]; // Fondo blanco por defecto
        Framebuffer {
            width,
            height,
            buffer,
            current_color: 0x000000, // Negro por defecto
        }
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn point(&mut self, x: isize, y: isize) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let index = (y as usize) * self.width + (x as usize);
            self.buffer[index] = self.current_color;
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }
}
