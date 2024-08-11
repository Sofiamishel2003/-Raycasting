use std::fs::File;
use std::io::{BufRead, BufReader};

/// Función para cargar un laberinto desde un archivo y almacenarlo en un array de dos dimensiones
pub fn load_maze(filename: &str) -> Vec<Vec<char>> {
    let file = File::open(filename).expect("No se pudo abrir el archivo.");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.expect("Error al leer la línea").chars().collect())
        .collect()
}


/// Función para imprimir el laberinto en la consola
pub fn print_maze(maze: &Vec<Vec<char>>) {
    for row in maze {
        for &cell in row {
            print!("{}", cell);
        }
        println!();
    }
}

fn main() {
    // Cargar el laberinto desde un archivo
    let maze = load_maze("C:/Users/50250/Desktop/Sofía Mishell Velásquez UVG/Tercer Año 2024/Segundo Semestre/Graficas/-Raycasting/maze.txt");

    // Imprimir el laberinto
    print_maze(&maze);
}
