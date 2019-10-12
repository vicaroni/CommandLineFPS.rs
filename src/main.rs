use std::{
    f64::consts::PI,
    ptr::null,
    time::SystemTime,
    mem};

use winapi::{
    shared::ntdef::NULL,
    um::{
        winnt::{GENERIC_READ, GENERIC_WRITE, LPCWSTR, WCHAR},
        wincon::{CreateConsoleScreenBuffer, SetConsoleActiveScreenBuffer, CONSOLE_TEXTMODE_BUFFER, WriteConsoleOutputCharacterW},
        winuser::GetAsyncKeyState,
        wincontypes::COORD}
};

fn get_async_key_state(c: char) -> u16{
    unsafe {mem::transmute(GetAsyncKeyState(c as i32))}
}

fn main() {
    const SCREEN_WIDTH: usize = 120;
    const SCREEN_HEIGHT: usize = 40;
    let mut screen: [WCHAR; SCREEN_WIDTH * SCREEN_HEIGHT] = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let h_console;
    unsafe{
        h_console = CreateConsoleScreenBuffer(GENERIC_READ | GENERIC_WRITE, 0, null(), CONSOLE_TEXTMODE_BUFFER, NULL);
        SetConsoleActiveScreenBuffer(h_console);}

    const FOV: f64 = 3.14159 / 4.0;
    const DEPTH: f64 = 16.0;
    const MAP_WIDTH: usize = 16;
    const MAP_HEIGHT: usize = 16;
    const MAP: &[u8] =
        b"#########.......\
         #...............\
         #.......########\
         #..............#\
         #......##......#\
         #......##......#\
         #..............#\
         ###............#\
         ##.............#\
         #......####..###\
         #......#.......#\
         #......#.......#\
         #..............#\
         #......#########\
         #..............#\
         ################";

    let (mut player_x, mut player_y, mut player_a, speed) = (14.5, 5.09, 0.0, 5.0);

    let mut tp1 = SystemTime::now();
    let mut tp2: SystemTime;
    loop{
        tp2 = SystemTime::now();
        let elapsed_time = tp2.duration_since(tp1).unwrap().as_secs_f64();
        tp1 = tp2;

        if get_async_key_state('A') & 0x8000 != 0{
            player_a -= speed * 0.75 * elapsed_time;
            if player_a < 0.{
                player_a += 2. * PI;
            }
        }
        if get_async_key_state('D') & 0x8000 != 0{
            player_a += speed * 0.75 * elapsed_time;
            if player_a >= 2. * PI{
                player_a -= 2. * PI;
            }
        }

        if get_async_key_state('W') & 0x8000 != 0{
            let (x, y) = (
                player_x + player_a.sin() * speed * elapsed_time,
                player_y + player_a.cos() * speed * elapsed_time);
            if MAP[(x as isize * MAP_WIDTH as isize + y as isize) as usize] != '#' as u8 {
                player_x = x;
                player_y = y;
            }
        }
        if get_async_key_state('S') & 0x8000 != 0{
            let (x, y) = (
                player_x - player_a.sin() * speed * elapsed_time,
                player_y - player_a.cos() * speed * elapsed_time);
            if MAP[(x as isize * MAP_WIDTH as isize + y as isize) as usize] != '#' as u8 {
                player_x = x;
                player_y = y;
            }
        }

        for x in 0..SCREEN_WIDTH{
            const STEP_SIZE: f64 = 0.1;
            let (rayAngle, mut distanceToWall, mut hitWall, mut boundary) = (
                player_a - FOV / 2. + x as f64 / SCREEN_WIDTH as f64 * FOV,
                0.0,
                false,
                false);
            let (eyeX, eyeY) = (rayAngle.sin(), rayAngle.cos());

            while !hitWall && distanceToWall < DEPTH {
                distanceToWall += STEP_SIZE;
                let (testX, testY) = (
                    (player_x + eyeX * distanceToWall) as isize,
                    (player_y + eyeY * distanceToWall) as isize);

                // Test if ray is out of bounds
                if testX < 0 || testX >= MAP_WIDTH as isize || testY < 0 || testY >= MAP_HEIGHT as isize {
                    hitWall = true;
                    distanceToWall = DEPTH;
                }  // Test if the ray cell is a wall block
                else if MAP[(testX * MAP_WIDTH as isize + testY) as usize] == '#' as u8 {
                    hitWall = true;

                    // To highlight tile boundaries, cast a ray from each corner
                    // of the tile, to the player. The more coincident this ray
                    // is to the rendering ray, the closer we are to a tile 
                    // boundary, which we'll shade to add detail to the walls
                    let mut p: Vec<(f64, f64)> = Vec::new();

                    // Test each corner of hit tile, storing the distance from
                    // the player, and the calculated dot product of the two rays
                    for tx in 0..2{
                        for ty in 0..2{
                            // Angle of corner to eye
                            let (vy, vx) = (
                                (testY + ty) as f64 - player_y,
                                (testX + tx) as f64 - player_x);
                            let d = (vx * vx + vy * vy).sqrt();
                            let dot = eyeX * vx / d + eyeY * vy / d;
                            p.push((d, dot));
                        }
                    }

                    p.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                    const BOUND: f64 = 0.01;
                    if p[0].1.acos() < BOUND ||
                       p[1].1.acos() < BOUND ||
                       p[2].1.acos() < BOUND {
                           boundary = true;
                    }
                }
            }

            let ceiling = (SCREEN_HEIGHT as f64 / 2. - SCREEN_HEIGHT as f64 / distanceToWall) as isize;
            let floor = SCREEN_HEIGHT as isize - ceiling;

            let shade =
                if boundary{' ' as WCHAR}
                else if distanceToWall <= DEPTH / 4. {0x2588}
                else if distanceToWall < DEPTH / 3. {0x2593}
                else if distanceToWall < DEPTH / 2. {0x2592}
                else if distanceToWall < DEPTH {0x2591}
                else{' ' as WCHAR};

            for y in 0..SCREEN_HEIGHT{
                screen[y * SCREEN_WIDTH + x] =
                    if y as isize <= ceiling{' ' as WCHAR}
                    else if y as isize > ceiling && y as isize <= floor{shade}
                    else{
                        let b = 1. - (y - SCREEN_HEIGHT / 2) as f64 / (SCREEN_HEIGHT / 2) as f64;
                        if b < 0.25{'#' as WCHAR}
                        else if b < 0.5{'x' as WCHAR}
                        else if b < 0.75{'.' as WCHAR}
                        else if b < 0.9{'-' as WCHAR}
                        else{' ' as WCHAR}
                    };
            }
        }

        let stat: Vec<u16> = format!("X={:.2}, Y={:.2}, A={:.2} FPS={:.2} ", player_x, player_y, player_a.to_degrees(), 1. / elapsed_time).encode_utf16().collect();
        &mut screen[..stat.len()].copy_from_slice(&stat);

        for nx in 0..MAP_WIDTH{
            for ny in 0..MAP_WIDTH{
                screen[(ny + 1) * SCREEN_WIDTH + nx] = MAP[ny * MAP_WIDTH + nx].into();
            }
        }
        screen[(player_x as usize + 1) * SCREEN_WIDTH + player_y as usize] = {
            let a = player_a.to_degrees();
            if a >= 315. || a < 45. {'>'}
            else if a >= 45. && a < 135. {'v'}
            else if a >= 135. && a < 225. {'<'}
            else {'^'}} as WCHAR;

        screen[SCREEN_WIDTH * SCREEN_HEIGHT - 1] = '\0' as WCHAR;
        let mut n: u32 = 0;
        unsafe{WriteConsoleOutputCharacterW(h_console, &screen as LPCWSTR, (SCREEN_WIDTH * SCREEN_HEIGHT) as u32, COORD{X: 0, Y: 0}, &mut n as *mut u32);}
    }
}
