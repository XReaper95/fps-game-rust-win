use std::ptr;
use std::f64::consts::PI;
use std::time::Instant;

// Windows API for handling console
use winapi::ctypes::wchar_t;
use winapi::shared::ntdef::NULL;
use winapi::um::wincon::{
    CreateConsoleScreenBuffer, SetConsoleActiveScreenBuffer, SetConsoleScreenBufferSize,
    WriteConsoleOutputCharacterW, CONSOLE_TEXTMODE_BUFFER, COORD,
};
use winapi::um::winuser::GetAsyncKeyState;
use winapi::um::winnt::{GENERIC_READ, GENERIC_WRITE, HANDLE, SHORT};

const SCREEN_WIDTH: u32 = 120;
const SCREEN_HEIGHT: u32 = 40;
const MAP_WIDTH: u32 = 16;
const MAP_HEIGHT: u32 = 16;
const MAX_DEPTH: f64 = 16.0;
const PLAYER_FOV: f64 = PI / 4.0;
const SPEED: f64 = 5.0;

fn is_wall(map: &[u8], pos_x: f64, pos_y: f64) -> bool {
    let index = pos_y as u32 * MAP_WIDTH + pos_x as u32;
    map[index as usize] == '#' as u8
}

fn swprintf_s(buffer: &mut [wchar_t], text: &str, mut line: u32){
    if line > SCREEN_HEIGHT - 1{
        line = SCREEN_HEIGHT - 1;
    }

    let console_line_index = (line * SCREEN_WIDTH) as usize;

    for (i, character) in text.chars().enumerate(){
        buffer[console_line_index + i] = character as wchar_t;
    }
}

fn main() {
    let mut player_x = 8.0; //middle of room
    let mut player_y = 8.0; //middle of room
    let mut player_a = 0.0;

    let mut map: String = String::new();
    map.push_str("################");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("#..............#");
    map.push_str("################");
    let map_array = map.as_bytes();

    let mut screen = [' ' as wchar_t; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
    let console_buffer: HANDLE;
    let mut dw_bytes_writen = 0;

    unsafe {
        console_buffer = CreateConsoleScreenBuffer(
            GENERIC_READ | GENERIC_WRITE,
            0,
            ptr::null(),
            CONSOLE_TEXTMODE_BUFFER,
            NULL,
        );

        SetConsoleActiveScreenBuffer(console_buffer);

        eprintln!("HELLO WORLD");
    }

    let mut tp1 = Instant::now();
    #[allow(unused_assignments)]
    let mut tp2 = Instant::now();

    // main game loop
    loop {
        tp2 = Instant::now();
        let elapsed_time = tp2.duration_since(tp1).as_secs_f64();
        tp1 = tp2;

        // controls handling
        unsafe {
            if GetAsyncKeyState('A' as i32) != 0 {
                player_a -= SPEED * 0.75 * elapsed_time;
            }
            if GetAsyncKeyState('D' as i32) != 0 {
                player_a += SPEED * 0.75 * elapsed_time;
            }
            if GetAsyncKeyState('W' as i32) != 0 {
                player_x += player_a.sin() * SPEED * elapsed_time;
                player_y += player_a.cos() * SPEED * elapsed_time;

                if is_wall(&map_array, player_x , player_y){
                    player_x -= player_a.sin() * SPEED * elapsed_time;
                    player_y -= player_a.cos() * SPEED * elapsed_time;
                }
            }
            if GetAsyncKeyState('S' as i32) != 0 {
                player_x -= player_a.sin() * SPEED * elapsed_time;
                player_y -= player_a.cos() * SPEED * elapsed_time;

                if is_wall(&map_array, player_x , player_y) {
                    player_x += player_a.sin() * SPEED * elapsed_time;
                    player_y += player_a.cos() * SPEED * elapsed_time;
                }
            }
        }

        for col in 0..SCREEN_WIDTH {
            // for each column calculate the ray angle into world space
            let ray_angle = (player_a - PLAYER_FOV / 2.0) + (col as f64 / SCREEN_WIDTH as f64)  * PLAYER_FOV;

            let mut wall_distance = 0.0;
            let mut hit_wall = false;
            let mut boundary = false;

            let eye_x = ray_angle.sin(); // unit vector for ray in player space
            let eye_y =  ray_angle.cos();

            while !hit_wall && wall_distance < MAX_DEPTH {
                wall_distance += 0.1;

                let test_x = (player_x + eye_x * wall_distance) as u32;
                let test_y = (player_y + eye_y * wall_distance) as u32;

                // check ray out of bounds
                if test_x as u32 >= MAP_WIDTH || test_y as u32 >= MAP_HEIGHT {
                    wall_distance = MAX_DEPTH;
                    hit_wall = true;
                } else {
                    if is_wall(&map_array, test_x as f64, test_y as f64) {
                        hit_wall = true;
                        let mut p: Vec<(f64, f64)> = Vec::new();

                        for tx in 0..2 {
                            for ty in 0..2 {
                                let vy = (test_y + ty) as f64 - player_y;
                                let vx = (test_x + tx) as f64 - player_x;
                                let d = (vx * vx + vy * vy).sqrt();
                                let dot = (eye_x * vx / d) + (eye_y * vy / d);
                                p.push((d, dot));
                            }
                        }

                        p.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                        let bound = 0.01;
                        if (p[0].1).acos() < bound { boundary = true };
                        if (p[1].1).acos() < bound { boundary = true };
                        if (p[2].1).acos() < bound { boundary = true };
                    }
                }
            }

            //calculate distance to ceiling and floor
            let ceiling = (SCREEN_HEIGHT / 2) as i32 - (SCREEN_HEIGHT as f64 / wall_distance) as i32;
            let floor = SCREEN_HEIGHT as i32 - ceiling;

            let mut shade;

            for row in 0..SCREEN_HEIGHT as i32{
                let index = (row * SCREEN_WIDTH as i32 + col as i32) as usize;

                if row < ceiling{
                    screen[index] = ' ' as wchar_t;
                }
                else if row > ceiling && row <= floor{
                    if wall_distance <= MAX_DEPTH as f64 / 4.0      {shade = 0x2588;} // very close
                    else if wall_distance <= MAX_DEPTH as f64 / 3.0 {shade = 0x2593;}
                    else if wall_distance <= MAX_DEPTH as f64 / 2.0 {shade = 0x2592;}
                    else if wall_distance <= MAX_DEPTH              {shade = 0x2591;}
                    else                                            {shade = ' ' as wchar_t;} // very far
                    if boundary                                     {shade = ' ' as wchar_t;}
                    screen[index] = shade;
                }
                else{
                    let b = 1.0 - (row as f64 - SCREEN_HEIGHT as f64 / 2.0) / (SCREEN_HEIGHT as f64 / 2.0);
                    if b < 0.25      {shade = '#' as wchar_t}
                    else if b < 0.5  {shade = 'x' as wchar_t}
                    else if b < 0.75 {shade = '.' as wchar_t}
                    else if b < 0.9  {shade = '-' as wchar_t}
                    else             {shade = ' ' as wchar_t}
                    screen[index] = shade;
                }
            }
        }

        // minimap (upper right)
        let minimap_offset = SCREEN_WIDTH - MAP_WIDTH;
        for row in 0..MAP_HEIGHT{
            for col in 0..MAP_WIDTH{
                let screen_index = (row * SCREEN_WIDTH + col + minimap_offset) as usize;
                let map_index = (row * MAP_WIDTH + col) as usize;
                screen[screen_index] = map_array[map_index] as wchar_t;
            }
            // player pos
            let player_index = player_y as u32 * SCREEN_WIDTH + player_x as u32 + minimap_offset;
            screen[player_index as usize] = '@' as wchar_t;
        }

        swprintf_s(&mut screen, format!("FPS: {:.0}", 1.0 / elapsed_time).as_str(), 0);
        swprintf_s(&mut screen, format!(
            "X: {:.2}, Y: {:.2}, Angle: {:.4}",
            player_x, player_y, player_a).as_str(), 1);

        unsafe {
            WriteConsoleOutputCharacterW(
                console_buffer,
                &screen[0],
                (SCREEN_WIDTH * SCREEN_HEIGHT) as u32,
                COORD { X: 0, Y: 0 },
                &mut dw_bytes_writen,
            );
        }
    }
}

