use std::u32;

use sdl2::{
    controller::Axis,
    pixels::Color,
    rect::{FPoint, Point, Rect},
    render::Canvas,
};

struct Player<'a> {
    x: f32,
    y: f32,
    fov: f32,
    rot: f32,
    controller: Option<&'a sdl2::controller::GameController>,
    rotdir: f32,
    forward_vel: f32,
    right_vel: f32,
    debug_dump: bool,
}

struct Map {
    map: Vec<Vec<usize>>,
    w: u32,
    h: u32,
}

#[derive(Clone, Copy)]
struct Coords(f32, f32);

struct Screen {
    screen: Canvas<sdl2::video::Window>,
    w: u32,
    h: u32,
    is_2d: bool,
}

fn draw_vertical_line(
    screen: &mut Screen,
    col: i32,
    top: i32,
    bottom: i32,
    color: sdl2::pixels::Color,
) {
    screen.screen.set_draw_color(color);
    screen
        .screen
        .draw_line(Point::new(col, top), Point::new(col, bottom))
        .unwrap();
}

// origin = (x: f32, y:32) tuple
// angle = f32 in radians
// cutoff = how far to cast the ray before giving up and returning None
// Returns Option<(coords: Coords, color: usize)>
fn raycast(origin: Coords, angle: f32, cutoff: f32, target_geo: &Map) -> Option<(Coords, usize)> {
    let mut dir_x = 0.0;
    let mut dir_y = 0.0;
    if angle > 1.5707964 && angle <= 3.1415927 {
        dir_x = -1.0;
    } else if angle > 3.1415927 && angle <= 4.712389 {
        dir_x = -1.0;
        dir_y = -1.0;
    } else if angle > 4.712389 && angle <= 6.2831855 {
        dir_y = -1.0;
    }
    let dir_x = dir_x;
    let dir_y = dir_y;

    let angle_tan = angle.tan();
    let angle_cot = 1.0 / angle_tan;
    let mut next_x_intersection = origin;
    next_x_intersection.0 += dir_x;
    next_x_intersection.0 = next_x_intersection.0.ceil();
    // delta_x is actually change in y for the closest position to origin which x % 1.0 == 0
    let delta_x = (next_x_intersection.0 - origin.0) * angle_tan;
    next_x_intersection.1 = delta_x + origin.1;

    let mut next_y_intersection = origin;
    next_y_intersection.1 += dir_y;
    next_y_intersection.1 = next_y_intersection.1.ceil();
    // delta_y is actually change in x for the closest position to origin which y % 1.0 == 0
    let delta_y = (next_y_intersection.1 - origin.1) * angle_cot;
    next_y_intersection.0 = delta_y + origin.0;

    while (((next_x_intersection.1 - origin.1) * (next_x_intersection.1 - origin.1)
        + (next_x_intersection.0 - origin.0) * (next_x_intersection.0 - origin.0))
        <= cutoff * cutoff)
        || (((next_y_intersection.0 - origin.0) * (next_y_intersection.0 - origin.0)
            + (next_y_intersection.1 - origin.1) * (next_y_intersection.1 - origin.1))
            <= cutoff * cutoff)
    {
        // get closer of the two ray options
        if ((next_x_intersection.1 - origin.1) * (next_x_intersection.1 - origin.1)
            + (next_x_intersection.0 - origin.0) * (next_x_intersection.0 - origin.0))
            <= ((next_y_intersection.0 - origin.0) * (next_y_intersection.0 - origin.0)
                + (next_y_intersection.1 - origin.1) * (next_y_intersection.1 - origin.1))
        {
            // next_x_intersection is closer
            // just bounds check first, it'll be too annoying and ugly to use .get().is_some() or whatever for a 2d array
            if (next_x_intersection.1 as u32) < target_geo.h && next_x_intersection.1 as u32 > 0 {
                if (next_x_intersection.0 as u32) < target_geo.w && next_x_intersection.0 as u32 > 0
                {
                    // check if target_geo.map[y as usize][(x + dir_x) as usize] is something
                    //      if it is map geometry, that's a hit, return Some((ray's position, the value at the map geometry))
                    if target_geo.map[next_x_intersection.1 as usize]
                        [(next_x_intersection.0 + dir_x) as usize]
                        != 0
                    {
                        return Some((
                            next_x_intersection,
                            target_geo.map[next_x_intersection.1 as usize]
                                [(next_x_intersection.0 + dir_x) as usize],
                        ));
                    }
                }
            }
            // advance next_x_intersection to the next x % 1.0 == 0
            if dir_x < 0.0 {
                next_x_intersection.0 -= 1.0;
                next_x_intersection.1 -= angle_tan;
            } else {
                next_x_intersection.0 += 1.0;
                next_x_intersection.1 += angle_tan;
            }
        } else {
            // next_y_intersection is closer
            // just bounds check first, it'll be too annoying and ugly to use .get().is_some() or whatever for a 2d array
            if (next_y_intersection.1 as u32) < target_geo.h && next_y_intersection.1 as u32 > 0 {
                if (next_y_intersection.0 as u32) < target_geo.w && next_y_intersection.0 as u32 > 0
                {
                    // check if target_geo.map[(y + dir_y) as usize][x as usize] is something
                    //      if it is map geometry, that's a hit, return Some((ray's position, the value at the map geometry))
                    if target_geo.map[(next_y_intersection.1 + dir_y) as usize]
                        [next_y_intersection.0 as usize]
                        != 0
                    {
                        return Some((
                            next_y_intersection,
                            target_geo.map[(next_y_intersection.1 + dir_y) as usize]
                                [next_y_intersection.0 as usize],
                        ));
                    }
                }
            }
            // advance next_y_intersection to the next y % 1.0 == 0
            if dir_y < 0.0 {
                next_y_intersection.0 -= angle_cot;
                next_y_intersection.1 -= 1.0;
            } else {
                next_y_intersection.0 += angle_cot;
                next_y_intersection.1 += 1.0;
            }
        }
    }
    //  do both closest intersections exceed the cutoff distance? return None
    None
}

fn render3d(camera: &mut Player, geometry: &Map, screen: &mut Screen) {
    screen.screen.clear();
    screen.screen.set_draw_color(Color::RGB(50, 50, 50));
    screen
        .screen
        .fill_rect(Rect::new(0, 0, screen.w, screen.h / 2))
        .unwrap();
    screen.screen.set_draw_color(Color::RGB(20, 20, 20));
    screen
        .screen
        .fill_rect(Rect::new(0, (screen.h / 2) as i32, screen.w, screen.h / 2))
        .unwrap();

    // screen info
    const COLORS: [sdl2::pixels::Color; 4] = [Color::RED, Color::YELLOW, Color::BLUE, Color::GREEN];
    let fov: f32 = camera.fov;
    const RENDER_DIST: f32 = 17.0;

    (0..screen.w).for_each(|i| {
        let mut angle = camera.rot + fov * 0.5 * ((i as f32 / screen.w as f32) * 2.0 - 1.0);
        if angle >= 360.0 {
            angle -= 360.0;
        } else if angle < 0.0 {
            angle += 360.0;
        }

        if let Some((hit, color)) = raycast(
            Coords(camera.x, camera.y),
            angle.to_radians(),
            RENDER_DIST,
            geometry,
        ) {
            let dist = ((hit.0 - camera.x) * (hit.0 - camera.x)
                + (hit.1 - camera.y) * (hit.1 - camera.y))
                .sqrt();

            let height = screen.h as f32 / dist;

            let top = ((screen.h as i32) / 2 - height as i32).max(0);
            let bottom = screen.h as i32 - top;
            if camera.debug_dump {
                println!("Height `{height}`, Top `{top}` at ray {i} with angle {angle}");
            }
            draw_vertical_line(screen, i as i32, top, bottom, COLORS[color - 1]);
        }
    });
    screen.screen.present();
    camera.debug_dump = false;
}

fn render2d(camera: &mut Player, geometry: &Map, screen: &mut Screen) {
    screen.screen.clear();
    screen.screen.set_draw_color(Color::RGB(50, 50, 50));
    screen
        .screen
        .fill_rect(Rect::new(0, 0, screen.w, screen.h))
        .unwrap();

    // screen info
    const COLORS: [sdl2::pixels::Color; 4] = [Color::RED, Color::YELLOW, Color::BLUE, Color::GREEN];
    let fov: f32 = camera.fov;
    const RENDER_DIST: f32 = 20.0;
    // const CLOSE: f32 = 1.0;

    let unit_x = screen.w / geometry.w;
    let unit_y = screen.h / geometry.h;

    for (y, i) in geometry.map.iter().enumerate() {
        for (x, cell) in i.iter().enumerate() {
            if *cell != 0 {
                screen.screen.set_draw_color(COLORS[cell - 1]);
                screen
                    .screen
                    .fill_rect(Rect::new(
                        (x as u32 * unit_x) as i32,
                        (y as u32 * unit_y) as i32,
                        unit_x,
                        unit_y,
                    ))
                    .unwrap();
            }
        }
    }

    let rays: Vec<(FPoint, usize)> = (0..screen.w).fold(Vec::new(), |mut acc, i| {
        let mut angle = camera.rot + fov * 0.5 * ((i as f32 / screen.w as f32) * 2.0 - 1.0);
        if angle >= 360.0 {
            angle -= 360.0;
        } else if angle < 0.0 {
            angle += 360.0;
        }

        let tmp =
            1.0 / ((fov * 0.5).to_radians() * ((i as f32 / screen.w as f32) * 2.0 - 1.0)).cos();
        // let near = tmp * CLOSE;
        let far = tmp * RENDER_DIST;
        if camera.debug_dump {
            println!("Far: `{far}` at ray {i}");
        }

        if let Some(hit) = raycast(
            Coords(camera.x, camera.y),
            angle.to_radians(),
            far,
            geometry,
        ) {
            acc.push((
                FPoint::new(hit.0 .0 * unit_x as f32, hit.0 .1 * unit_y as f32),
                hit.1,
            ));
        } else {
            acc.push((
                FPoint::new(
                    (camera.x + angle.to_radians().cos() * far) * unit_x as f32,
                    (camera.y + angle.to_radians().sin() * far) * unit_y as f32,
                ),
                4,
            ));
        }
        acc
    });

    for (ray_end, color) in rays {
        screen.screen.set_draw_color(COLORS[color - 1]);
        screen
            .screen
            .draw_fline(
                FPoint::new(camera.x * unit_x as f32, camera.y * unit_y as f32),
                ray_end,
            )
            .unwrap();
    }
    screen.screen.present();
    camera.debug_dump = false;
}

impl<'a> Player<'a> {
    fn update(&mut self, delta_time: u32, geometry: &Map) {
        if let Some(controller) = self.controller.clone() {
            let val = controller.axis(Axis::RightX);
            if !(val > -2000 && val < 2000) {
                self.rot += val as f32 * 0.0003 * 20.0 * 0.001 * delta_time as f32;
            }
        }
        if self.rotdir != 0.0 {
            self.rot += self.rotdir * 0.0003 * 20.0 * 0.001 * delta_time as f32;
        }

        if let Some(controller) = self.controller.clone() {
            let val_y = controller.axis(Axis::LeftY);
            let val_x = controller.axis(Axis::LeftX);
            if !(val_y > -3000 && val_y < 3000) {
                self.x -= self.rot.to_radians().cos()
                    * val_y as f32
                    * 0.00008
                    * 0.001
                    * delta_time as f32;
                self.y -= self.rot.to_radians().sin()
                    * val_y as f32
                    * 0.00008
                    * 0.001
                    * delta_time as f32;
            }
            if !(val_x > -3000 && val_x < 3000) {
                self.x += (self.rot + 90.0).to_radians().cos()
                    * val_x as f32
                    * 0.00008
                    * 0.001
                    * delta_time as f32;
                self.y += (self.rot + 90.0).to_radians().sin()
                    * val_x as f32
                    * 0.00008
                    * 0.001
                    * delta_time as f32;
            }
        }
        if self.forward_vel != 0.0 {
            // new x = old x + (velocity*delta_time*cos(rot.to_rad()))
            self.x += self.rot.to_radians().cos()
                * self.forward_vel
                * 0.00008
                * 0.001
                * delta_time as f32;
            self.y += self.rot.to_radians().sin()
                * self.forward_vel
                * 0.00008
                * 0.001
                * delta_time as f32;
        }
        if self.right_vel != 0.0 {
            self.x += (self.rot + 90.0).to_radians().cos()
                * self.right_vel
                * 0.00008
                * 0.001
                * delta_time as f32;
            self.y += (self.rot + 90.0).to_radians().sin()
                * self.right_vel
                * 0.00008
                * 0.001
                * delta_time as f32;
        }

        // if character rotation exceeds 360, go ahead and cut 360 degrees from it
        if self.rot >= 360.0 {
            self.rot -= 360.0;
        } else if self.rot < 0.0 {
            self.rot += 360.0;
        }
        // and keep the player in bounds
        self.x = self.x.clamp(0.0, geometry.w as f32);
        self.y = self.y.clamp(0.0, geometry.h as f32);
    }
}

fn main() -> Result<(), String> {
    // init sdl and make a window
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let timer_subsystem = sdl_context.timer()?;
    let window = video_subsystem
        .window("Raycaster", 900, 640)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let canvas = window.into_canvas().build().unwrap();

    let mut screen = Screen {
        screen: canvas,
        w: 900,
        h: 640,
        is_2d: false,
    };

    // obtain event pump
    let mut event_pump = sdl_context.event_pump()?;

    // init game controller subsystem
    let game_controller_subsystem = sdl_context.game_controller()?;
    let controllers: Vec<(u32, String)> = (0..game_controller_subsystem.num_joysticks()?)
        .filter(|&i| game_controller_subsystem.is_game_controller(i))
        .map(|i| (i, game_controller_subsystem.name_for_index(i).unwrap()))
        .collect();

    // let mut default_controller = None;
    let mut current_controller = None;
    if !controllers.is_empty() {
        game_controller_subsystem.set_event_state(true);
        // default_controller = Some(controllers[0].clone());
        current_controller = Some(game_controller_subsystem.open(controllers[0].0).unwrap());
    }

    let map = Map {
        map: vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],
            vec![1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],
            vec![1, 0, 0, 0, 0, 2, 2, 0, 3, 3, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
        ],
        w: 20,
        h: 17,
    };

    let mut player = Player {
        x: 8.5,
        y: 8.5,
        rot: 0.0,
        fov: 75.0,
        controller: current_controller.as_ref(),
        rotdir: 0.0,
        forward_vel: 0.0,
        right_vel: 0.0,
        debug_dump: false,
    };

    let mut time = timer_subsystem.ticks();
    'running: loop {
        for event in event_pump.poll_iter() {
            use sdl2::{event::Event, keyboard::Keycode};
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    player.rotdir = i16::MIN as f32;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    player.rotdir = i16::MAX as f32;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if player.rotdir < 0.0 {
                        player.rotdir = 0.0;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    if player.rotdir > 0.0 {
                        player.rotdir = 0.0;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    player.forward_vel = i16::MAX as f32;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    player.forward_vel = i16::MIN as f32;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    if player.forward_vel > 0.0 {
                        player.forward_vel = 0.0;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if player.forward_vel < 0.0 {
                        player.forward_vel = 0.0;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    player.right_vel = i16::MAX as f32;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    player.right_vel = i16::MIN as f32;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    if player.right_vel < 0.0 {
                        player.right_vel = 0.0;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    if player.right_vel > 0.0 {
                        player.right_vel = 0.0;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    player.debug_dump = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    screen.is_2d = !screen.is_2d;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => {
                    player.fov -= 5.0;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    ..
                } => {
                    player.fov += 5.0;
                }
                _ => {}
            }
        }

        player.update(timer_subsystem.ticks() - time, &map);
        time = timer_subsystem.ticks();
        if screen.is_2d == true {
            render2d(&mut player, &map, &mut screen);
        } else {
            render3d(&mut player, &map, &mut screen);
        }
    }
    Ok(())
}
