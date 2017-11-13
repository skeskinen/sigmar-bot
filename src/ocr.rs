
use image::{ImageBuffer, Rgba, Rgb, Luma, ConvertBuffer};
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::cmp::{max, min};

use sigmar::{board_rows, Board, Marble};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SRGB {
    r: u8,
    g: u8,
    b: u8,
}

impl SRGB {
    fn eucl_dist(&self, other: &SRGB)-> f32 {
        let SRGB{r: r_s, g: g_s, b: b_s} = *self;
        let SRGB{r: r_o, g: g_o, b: b_o} = *other;
        let f = |a: u8, b: u8| -> f32 {let diff = a as f32 - b as f32; diff * diff};
        (f(r_s, r_o) + f(g_s, g_o) + f(b_s, b_o)).sqrt()
    }
}

macro_rules! pixel_arr {
    ($(($r:expr, $g: expr, $b: expr)),*) => {
        [ $( SRGB{r: $r, g: $g, b: $b},)* ]
    };
}
const GOLD_PIXEL_VALUES: [[SRGB;5];3] = [
    pixel_arr![(166, 147, 112), (189, 174, 139), (196, 181, 147), (194, 181, 147), (184, 168, 135)],
    pixel_arr![(161, 142, 109), (179, 161, 126), (194, 178, 142), (194, 177, 142), (175, 156, 123)],
    pixel_arr![(166, 147, 112), (163, 143, 110), (170, 150, 116), (168, 149, 115), (161, 141, 108)],
];

#[derive(Debug, Clone, Copy)]
struct RGB {
    r: f32,
    g: f32,
    b: f32,
}

impl From<SRGB> for RGB {
    fn from(SRGB{r, g, b}: SRGB) -> RGB {
        let to_linear = |c: u8| {
            let c_f32 = c as f32;
            if c_f32 <= 0.04045 {c_f32 / 12.92}
                else {((c_f32 + 0.055) / 1.055).powf(2.4)}
        };

        RGB{r: to_linear(r), g: to_linear(g), b: to_linear(b)}
    }
}

#[derive(Debug, Clone, Copy)]
struct GreyScale (f32);

impl From<GreyScale> for SRGB {
    fn from(GreyScale(y): GreyScale) -> SRGB {
        let c = if y <= 0.0031308 {12.92 * y} else {(1.055 * y).powf(1.0/2.4) - 0.055};
        let uint = c.round() as u8;
        SRGB{r: uint, g: uint, b: uint}
    }
}

fn to_grayscale(pixel: SRGB) -> GreyScale {
    let RGB{r,g,b} = RGB::from(pixel);
    GreyScale(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

struct Image<T> {
    w: usize,
    h: usize,
    data: Vec<T>,
}

impl<T> ::std::ops::Index<usize> for Image<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx]
    }
}

const TILE_WIDTH: f32 = 66.0;
const TILE_HEIGHT: f32 = 57.0;

fn recognize_marble_at(desktop_image: &Image<SRGB>, x: i32, y: i32) -> Marble {
    let mut data: Vec<SRGB> = Vec::with_capacity(30*40);
    for my in -20..20 {
        for mx in -15..15 {
            data.push(desktop_image[(x + mx) as usize + (y+my) as usize * desktop_image.w]);
        }
    }
    // let gray_marble = Image{w: 32, h: 40, data: data.iter().map(|p| to_grayscale(*p)).collect()};
    // let luma: Vec<f32> = gray_marble.data.iter().map(|g| g.0).collect();

    // let srgb = luma.iter().flat_map(|l| ::std::iter::repeat(SRGB::from(GreyScale(*l)).r).take(3)).collect();

    let mut asd = Vec::with_capacity(30*40*3);
    for d in data { asd.push(d.r); asd.push(d.g); asd.push(d.b)}

    let buf: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_vec(30, 40, asd).unwrap();

    buf.save(format!("images/{}_{}.png", x, y)).expect("save failed");

    Marble::Empty
}

fn capture(mut capturer: Capturer) -> Vec<u8> {
    loop {
        match capturer.frame() {
            Ok(buffer) => {
                let mut v = vec![0; buffer.len()];
                v.copy_from_slice(&(*buffer));
                return v;
            },
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep( Duration::new(0, 500_000));
                    continue;
                } else {
                    panic!("Capture error: {}", error);
                }
            }
        };
    }
}

pub fn ocr_game_board() -> Option<i32>{
    let display = Display::primary().expect("Couldn't find primary display.");
    let capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    let mut buffer = capture(capturer);

    _save_screenshot(&buffer, w, h);

    let desktop_image: Image<SRGB> = Image{
        w: w, h: h,
        data: (&buffer[..]).chunks(4).map(|chunk: &[u8]| {
                match chunk {
                    &[b, g, r, a] => SRGB{r: r, g: g, b: b},
                    _ => unreachable!()
                }
            }).collect()
    };

    let mut best_dist = ::std::f32::MAX;
    let mut best_coord = (0,0);

    for y in 10..(h-10) {
        for x in 10..(w-10) {
            let mut d: f32 = 0.0;
            for my in 0..3 {
                for mx in 0..5 {
                    let index = (y + my - 1) * w + (x + mx - 2);
                    d += desktop_image[index].eucl_dist(&GOLD_PIXEL_VALUES[my][mx]);
                }
            }
            if d < best_dist {
                best_dist = d;
                best_coord = (x as i32, y as i32);
            }
        }
    }

    if best_dist > 100.0 {
        return None;
    }

    println!("Best guess for board center {} {:?}", best_dist, best_coord);
    let (gold_x, gold_y) = best_coord;

    let rows = board_rows();

    let mut board: Board = [[Marble::Empty;13];13];

    for (i, r) in rows.iter().enumerate() {
        for x in r.x_min .. r.x_max + 1 {
            let (screen_x, screen_y) = board_pos_to_screen(x - 5, (i as i32 - 5));
            let (coord_x, coord_y) = (gold_x + (screen_x * TILE_WIDTH) as i32 + 1, gold_y + (screen_y * TILE_HEIGHT) as i32);
            
            board[i + 1][x as usize + 1] = recognize_marble_at(&desktop_image, coord_x, coord_y);
        }
    }

    Some(0)
}

fn _save_screenshot(buffer: &Vec<u8>, w: usize, h: usize) {
    let mut bitflipped = Vec::with_capacity(w * h * 4);
    for pixel in buffer.chunks(4) {
        let (b, g, r, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        bitflipped.extend_from_slice(&[r, g, b, a]);
    }

    let path = Path::new("screenshot.png");
    let image: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(
            w as u32,
            h as u32,
            bitflipped
        ).expect("Couldn't convert frame into image buffer.");

    image.save(&path).expect("Couldn't save image to `screenshot.png`.");
    println!("Image saved to `screenshot.png`.");
}


fn board_pos_to_screen(x: i32, y: i32) -> (f32, f32) {
    (x as f32 + (y as f32 / 2.0), -y as f32)
}
