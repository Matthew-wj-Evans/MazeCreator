use std::time::{Duration, Instant};
use std::io::Write;
use std::fs::File;
use image::RgbImage;
use rand::Rng;

const OUTPUT_PATH:&'static          str = "target/debug/output/";
const OUTPUT_FILE_ASCII:&'static    str = "ascii.txt";
const OUTPUT_FILE_PATH:&'static     str = "path.txt";
const OUTPUT_FILE_PNG:&'static      str = "maze.png";

const MAZE_PATH_CHAR:&'static       str = " ";
const MAZE_WALL_CHAR:&'static       str = "#";

const PIXEL_HEIGHT:                 u32 = 25; // pixels
const PIXEL_WIDTH:                  u32 = 25; // pizels
const PATH_HEIGHT:                  u32 = 8;
const PATH_WIDTH:                   u32 = 8;

const BLACK:                        [u8;3] = [0,0,0];
const WHITE:                        [u8;3] = [255,255,255];

#[derive(Copy, Clone, Debug)]
enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST,
    NONE,
} impl Direction {
    fn as_string(&self) -> &'static str {
        match self {
            Direction::NORTH => "N",
            Direction::EAST => "E",
            Direction::SOUTH => "S",
            Direction::WEST => "W",
            Direction::NONE => " ",
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let backtrack :Container = Container { 
        width: (PATH_WIDTH),
        height: (PATH_HEIGHT) 
    };
    let cell :Container = Container { 
        width: (PIXEL_WIDTH),  
        height: (PIXEL_HEIGHT) 
    };
    let maze: Container = Container { 
        width: (translate(backtrack.width,2,1)),    
        height: (translate(backtrack.height, 2, 1)),
    };

    // A boolean table to check if a coordinate has been traversed.
    let mut visited: Vec<bool> = vec![false; (backtrack.get_area()) as usize];

    // Stores the traversed path. Is only a size of zero twice, when it's started and when it's done.
    let mut stack_position: Stack<Coordinate> =  Stack::new(); 

    // Stores a coordinate and reference to the kind of transformation (North, East, South, West) to the next coordinate
    let mut paths: Vec<Path> = Vec::new(); 

    // Start off the sequence pushing the origin
    let start_coord = Coordinate {
        x: rng.gen_range(0..(backtrack.width - 1)),
        y: rng.gen_range(0..(backtrack.height - 1)),
    };
    stack_position.push(start_coord);
    // Mark the coordinate as visited
    update_visited(get_linear_coord(stack_position.peek().unwrap(), backtrack.width), &mut visited);

    println!("Starting backtracking...");
    let timer = get_timer();
    
    // Start backtracking
    while !stack_position.is_empty() {
        // peek at the end of the stack to get the coord
        // Get list of possible translations and the associated direction
        let coord = stack_position.peek().unwrap();
        // The tuple returned is the new coord and what direction the current needs to go
        let mut next_set: Vec<(Coordinate, Direction)> = valid_moves(*coord,backtrack, &visited);
        let mut update:bool = false;

        if next_set.len() > 0 {
            // No need to pop from the stack
            update = true;
        } else {
            // IMPORTANT:: Need to capture the first pop of a series, it's the end of a branch
            push_stack_end(stack_position.pop().unwrap(), &mut paths);
            next_set = get_set_from_stack(backtrack, &visited, &mut stack_position);

            if !next_set.is_empty() {
                update = true;
            }
        } // End if/else
        
        // If a valid coord was grabbed, push it to the stack, get the next coord, then update visisted with new coord
        if update {      
            let index:usize = rng.gen_range(0..next_set.len());
            update_state(next_set[index], &mut stack_position, &mut paths);
            update_visited(get_linear_coord(stack_position.peek().unwrap(), backtrack.width), &mut visited);
        }
    } 
    // End backtracking

    // Get the duration of the timer
    let duration_backtracking = get_duration(timer); 
    println!("Backtracking took {:.4} seconds.\n", duration_backtracking.as_secs_f32());
    
    println!("Starting to build the wireframe...");
    let timer = get_timer();
    let wireframe:WireFrame = create_wireframe(&paths, maze);
    let duration_wireframe = get_duration(timer);
    println!("Wireframe building took {:.4} seconds.\n", duration_wireframe.as_secs_f32());
    output_path(&paths); // returns the result of writing to file
    output_ascii(&wireframe);
    //convert_wireframe_ascii(&mut wireframe);
    //output_ascii_maze(&wireframe);

    println!("Starting to draw the png...");
    let timer = get_timer();
    draw_png(wireframe, cell, maze);
    let duration_drawing = get_duration(timer);
    println!("Drawing the png took {:.4} seconds.\n", duration_drawing.as_secs_f32());
}

fn push_stack_end(branch_end:Coordinate, paths:&mut Vec<Path>) {
    paths.push(Path { coordinate: (branch_end), direction: (Direction::NONE) });
}

fn update_state(next:(Coordinate,Direction), stack:&mut Stack<Coordinate>, paths:&mut Vec<Path>) {
    let coord = stack.peek().unwrap();
    let path:Path = Path { coordinate: (*coord), direction: (next.1) }; // push current coord w/ direction to path
    let coord = next.0; // set coord to equal its directional translation
    stack.push(coord);
    paths.push(path);
}

fn get_set_from_stack(backtrack:Container, visited:&Vec<bool>, stack:&mut Stack<Coordinate>) -> Vec<(Coordinate, Direction)> {
    // Get a new coord
    let coord: &Coordinate = stack.peek().unwrap();
    let mut next_set:Vec<(Coordinate, Direction)> = valid_moves(*coord, backtrack, &visited);

    // check if there are valid moves, if not start poping until there are OR the stack is empty
    while next_set.is_empty() && !stack.is_empty() {
        stack.pop(); // pop from mem
        if !stack.is_empty() {
            let coord: &Coordinate = stack.peek().unwrap();
            next_set = valid_moves(*coord, backtrack, &visited);
        }
    }
    next_set
}

fn valid_moves(coord:Coordinate, backtrack:Container, visited: &Vec<bool>) -> Vec<(Coordinate, Direction)> {
    let mut valid_moves: Vec<(Coordinate, Direction)> = Vec::new();
    let linear_index = get_linear_coord(&coord, backtrack.width);

    // Test a north translation, possible negative overflow, (1,1) -> (1,0)
    if !will_overflow(coord.y, 1) {
        if !visited[linear_index - backtrack.width as usize] {
            valid_moves.push((Coordinate { x: (coord.x), y: (coord.y - 1) }, Direction::NORTH));
        }
    }

    // Test a west translation, possible negative overflow, (1,1) -> (0,1)
    if !will_overflow(coord.x, 1) {
        if !visited[linear_index - 1] {
            valid_moves.push((Coordinate { x: (coord.x - 1), y: (coord.y) }, Direction::WEST));
        }
    }
    // Test an east translation (1,1) -> (2,1)
    if coord.x + 1 < backtrack.width {
        if !visited[linear_index+1] {
            valid_moves.push((Coordinate { x: (coord.x + 1), y: (coord.y) }, Direction::EAST));
        }
    }

    // Test a south translation (1,1) -> (1,2)
    if coord.y + 1 < backtrack.height {
        if !visited[linear_index+backtrack.width as usize] {
            valid_moves.push((Coordinate { x: (coord.x), y: (coord.y + 1) }, Direction::SOUTH));
        }
    }

    valid_moves
}

fn translate(value:u32, coefficient:u32, offset:u32) -> u32 {
    return coefficient * value + offset;
}

fn will_overflow(value:u32, offset:u32) -> bool {
    return value < offset 
}

fn update_visited(coord:usize, visited:&mut Vec<bool>) {
    visited[coord] = true;
}

fn get_linear_coord(coord:&Coordinate, width:u32) -> usize {
    return (coord.y * width + coord.x) as usize
}

fn get_duration(instant:Instant) -> Duration {
    instant.elapsed()
}

fn get_timer() -> Instant {
    let timer = Instant::now();
    timer
}

fn paint_square(square:Square, image:&mut RgbImage) {
    for y in 0..square.dimensions.height {
        for x in 0..square.dimensions.width {
            *image.get_pixel_mut((square.dimensions.width*square.start.x) + x, (square.dimensions.height*square.start.y) + y) = image::Rgb(square.colour);
        }
    }
}

fn draw_png(frame:WireFrame, cell:Container, maze:Container) {

    let mut image: RgbImage = RgbImage::new(maze.width*cell.width, maze.height*cell.height);
    for i in 0..(frame.data.len() as u32) {
        let x:u32 = i % frame.width;
        let y:u32 = (i - x) / frame.width;
        let square:Square = Square { 
            start: Coordinate { x:(x), y:(y)},
            dimensions: Container { width: (cell.width), height: (cell.height) },
            colour: if frame.data[i as usize] == MAZE_PATH_CHAR {WHITE} else {BLACK},
        };
        paint_square(square, &mut image);
    }
    let path = String::from(OUTPUT_PATH);
    let path = path + OUTPUT_FILE_PNG;
    image.save(path).unwrap();
}

fn create_wireframe(paths:&Vec<Path>, maze:Container) -> WireFrame {
    let width = (maze.width) as usize;
    let height = (maze.height) as usize;
    let mut wireframe:Vec<&str> = vec![MAZE_WALL_CHAR; width*height];

    for path in paths {
        // Translate the coordinates
        let x = translate(path.coordinate.x, 2, 1) as usize;
        let y = translate(path.coordinate.y, 2, 1) as usize;
        let linear_index = y * width + x;
        let path_index = match path.direction {
            Direction::NORTH => linear_index - width,
            Direction::EAST => linear_index + 1,
            Direction::SOUTH => linear_index + width,
            Direction::WEST => linear_index - 1,
            Direction::NONE => linear_index,
        };

        wireframe[linear_index] = MAZE_PATH_CHAR;
        wireframe[path_index] = MAZE_PATH_CHAR;
    }
    let frame:WireFrame = WireFrame { width: (width.try_into().unwrap()), data: (wireframe) };
    frame
}

fn output_ascii(frame:&WireFrame) {
    let mut output = String::new();
    for i in 0..frame.data.len() {
        output += frame.data[i];
        if i % frame.width as usize == frame.width as usize - 1 {
            output += "\n";
        }
    }
    let file_path = String::from(OUTPUT_PATH);
    let file_path = file_path + OUTPUT_FILE_ASCII;
    write_to_file(file_path, output);
}

fn output_path(paths:&Vec<Path>) {
    let mut output = String::new();
    for path in paths {
        output += (format!("{},{}-{}\n", path.coordinate.x, path.coordinate.y, path.direction.as_string())).as_str();
    }
    let file_path = String::from(OUTPUT_PATH);
    let file_path = file_path + OUTPUT_FILE_PATH;
    write_to_file(file_path, output);
}

fn write_to_file(path:String, output:String) {
    let mut file = File::create(path).unwrap();
    file.write_all(output.as_bytes()).unwrap(); 
}

struct Square {
    start: Coordinate,
    dimensions: Container,
    colour: [u8; 3],
}

#[derive(Clone, Debug)]
struct WireFrame {
    width:u32,
    data: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug)]
struct Coordinate {
    x: u32,
    y: u32,
}

#[derive(Clone, Copy, Debug)]
struct Path {
    coordinate: Coordinate,
    direction: Direction,
}

#[derive(Clone, Copy, Debug)]
struct Container {
    width: u32,
    height: u32,
} impl Container {
    fn get_area(&self) -> u32 {
        &self.width * &self.height
    }
}

#[derive(Debug)]
struct Stack<T> {
    stack: Vec<T>,
} impl<T> Stack<T> {
    fn new() -> Self {
        Stack {stack: Vec::new()}
    }

    fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }
    
    fn push(&mut self, item: T) {
        self.stack.push(item)
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn peek(&self) -> Option<&T> {
        self.stack.last()
    }
}