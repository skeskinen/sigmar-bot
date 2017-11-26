use std;
use image;
use image::{ImageBuffer, Rgba};
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::path::Path;
use std::thread;
use std::time::Duration;
use num;
use std::collections::VecDeque;
use std::error::Error;

use sigmar::{board_rows, Board, Marble, MARBLE_VALUES};

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
        let f = |a: u8, b: u8| -> f32 {let diff = f32::from(a) - f32::from(b); diff * diff};
        (f(r_s, r_o) + f(g_s, g_o) + f(b_s, b_o)).sqrt()
    }
}

struct SRGBIntoIter {
    srgb: SRGB,
    pos: u8,
}

impl<'a> Iterator for SRGBIntoIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        match self.pos {
            1 => Some(self.srgb.r),
            2 => Some(self.srgb.g),
            3 => Some(self.srgb.b),
            _ => None,
        }
    }
}

impl IntoIterator for SRGB {
    type Item = u8;
    type IntoIter = SRGBIntoIter;

    fn into_iter(self) -> SRGBIntoIter {
        SRGBIntoIter{srgb: self, pos: 0}
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
            let c_f32 = c as f32 / 255.0;
            if c_f32 <= 0.04045 {c_f32 / 12.92}
                else {((c_f32 + 0.055) / 1.055).powf(2.4)}
        };

        RGB{r: to_linear(r), g: to_linear(g), b: to_linear(b)}
    }
}

#[derive(Debug, Clone, Copy)]
struct Grayscale (f32);

impl From<Grayscale> for SRGB {
    fn from(Grayscale(y): Grayscale) -> SRGB {
        let c = if y <= 0.0031308 {12.92 * y} else {(1.055 * y).powf(1.0/2.4) - 0.055};
        let uint = (c * 255.0).round() as u8;
        SRGB{r: uint, g: uint, b: uint}
    }
}

impl From<RGB> for SRGB {
    fn from(RGB{r,g,b}: RGB) -> SRGB {
        let gamma_compress = |y: f32| -> u8 {
            let c = if y <= 0.0031308 {12.92 * y} else {(1.055 * y).powf(1.0/2.4) - 0.055};
            (c * 255.0).round() as u8
        };
        SRGB{r: gamma_compress(r), g: gamma_compress(g), b: gamma_compress(b)}
    }
}

impl From<EdgeGradient> for SRGB {
    fn from(edge: EdgeGradient) -> SRGB {
        let EdgeGradient { intensity, angle } = edge;

        let s = 1.0;
        let h = angle / std::f32::consts::PI * 3.0 + 3.0;
        let v = num::clamp(intensity * 2.0, 0.0, 1.0);

        let i = h.floor();
        let f = h - i;
        let p = v * ( 1.0 - s );
        let q = v * ( 1.0 - s * f );
        let t = v * ( 1.0 - s * ( 1.0 - f ) );

        let rgb = match i as i32 {
            0 => RGB{r: v, g: t, b: p},
            1 => RGB{r: q, g: v, b: p},
            2 => RGB{r: p, g: v, b: t},
            3 => RGB{r: p, g: q, b: v},
            4 => RGB{r: t, g: p, b: v},
            _ => RGB{r: v, g: p, b: q},
        };
        SRGB::from(rgb)
    }
}

fn to_grayscale(pixel: SRGB) -> Grayscale {
    let RGB{r,g,b} = RGB::from(pixel);
    Grayscale(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

#[derive(Clone, Debug)]
struct Image<T> {
    w: usize,
    h: usize,
    data: Vec<T>,
}

struct ImageIterMut<'a, T: 'a> {
    iter: std::slice::IterMut<'a, T>,
}

impl<'a, T: 'a> Iterator for ImageIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T> IntoIterator for &'a mut Image<T> {
    type Item = &'a mut T;
    type IntoIter = ImageIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ImageIterMut {
            iter: self.data.iter_mut(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EdgeGradient {
    intensity: f32,
    angle: f32,
}

fn gaussian_kernel(size: usize, std_dev: f32) -> Image<Grayscale> {
    let mut data = vec![Grayscale(0.0); size*size];

    let denom = std::f32::consts::PI * 2.0 * std_dev;

    let mut acc = 0.0;

    for (i, grey) in data.iter_mut().enumerate() {
        let x = (i % size) as i32;
        let y = (i / size) as i32;
        let half_size = (size / 2) as i32;
        let (x_dist, y_dist) = ((x - half_size).abs(), (y - half_size).abs());

        let Grayscale(ref mut f) = *grey;
        *f = (1.0 / denom) * (std::f32::consts::E.powf(- ((x_dist * x_dist) as f32 + (y_dist * y_dist) as f32) / (2.0 * std_dev)));
        acc += *f;
    }

    for grey in data.iter_mut() {
        let Grayscale(ref mut f) = *grey;
        *f = *f / acc;
    }

    Image {
        w: size,
        h: size,
        data: data,
    }
}

lazy_static! {
    static ref SOBEL_X: Image<Grayscale> = {
        Image {
            w: 3,
            h: 3,
            data: vec![1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0].iter().map(|&f| Grayscale(f)).collect(),
        }
    };
    static ref SOBEL_Y: Image<Grayscale> = {
        Image {
            w: 3,
            h: 3,
            data: vec![1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0].iter().map(|&f| Grayscale(f)).collect(),
        }
    };

    static ref GAUSS: Image<Grayscale> = {
        gaussian_kernel(5, 0.5)
    };
}

impl<T> ::std::ops::Index<usize> for Image<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx]
    }

}

impl Image<Grayscale> {
    fn convolute(&self, kernel: &Image<Grayscale>) -> Image<Grayscale> {
        let mut buf = vec![Grayscale(0.0); self.w * self.h];

        let val = |mut x: i32, mut y: i32| -> Grayscale {
            x = num::clamp(x, 0, (self.w - 1) as i32);
            y = num::clamp(y, 0, (self.h - 1) as i32);
            self.data[y as usize * self.w + x as usize]
        };

        for y in 0..self.h {
            for x in 0..self.w {
                let Grayscale(ref mut acc) = buf[y * self.w + x];
                let (kw, kh) = (kernel.w as i32, kernel.h as i32);
                let (half_kw, half_kh) = (kw/2, kh/2);
                for ky in 0..kh {
                    for kx in 0..kw {
                        let Grayscale(kernel_val) = kernel[(ky * kw + kx) as usize];
                        let Grayscale(image_val) = val(x as i32 + kx - half_kw, y as i32 + ky - half_kh);
                        *acc += kernel_val * image_val;
                    }
                }
            }
        }
        Image{w: self.w, h: self.h, data: buf}
    }

}

fn sobel(image: &Image<Grayscale>) -> Image<EdgeGradient> {
    let x = image.convolute(&SOBEL_X);
    let y = image.convolute(&SOBEL_Y);

    let data = x.data.iter()
        .zip(y.data.iter())
        .map(|(&Grayscale(x), &Grayscale(y))| EdgeGradient { 
            intensity: (x * x + y * y).sqrt(),
            angle: y.atan2(x),
        }).collect();

    Image {
        w: image.w,
        h: image.h,
        data: data,
    }
}

#[derive(PartialEq, Eq, Debug)]
enum CannyLabel {
    Strong,
    Weak,
    Suppressed,
}

fn canny(image: &Image<EdgeGradient>) -> Image<bool> {
    let mut max_intensity = 0.0;
    let non_maximum_suppressed: Vec<f32> = image.data.iter().enumerate().map(|(i, &EdgeGradient{angle, intensity})| {
        let x = (i % image.w) as i32;
        let y = (i / image.w) as i32;
        let get_neig = |a_x: i32, a_y: i32| -> f32 {
            let (t_x, t_y) = (x + a_x, y + a_y);
            if t_x < 0 || t_x >= image.w as i32 || t_y < 0 || t_y >= image.h as i32 { 0.0 }
            else { image.data[t_y as usize * image.w + t_x as usize].intensity }
        };
        let mut dir = (angle / std::f32::consts::PI * 4.0).round();
        if dir < 0.0 { dir = 4.0 + dir }
        let (d_x, d_y) = match dir as i32{
            0 => (1, 0),
            1 => (1, 1),
            2 => (0, 1),
            3 => (1, -1),
            4 => (0, 1),
            _ => panic!("canny bad angle"),
        };
        if get_neig(d_x, d_y) > intensity || get_neig(-d_x, -d_y) > intensity { 0.0 }
        else { 
            if intensity > max_intensity { max_intensity = intensity; }
            intensity
        }
    }).collect();

    let normalized: Vec<f32> = non_maximum_suppressed.iter().map(|&f| if max_intensity > 0.1 { f / max_intensity } else { 0.0 }).collect();

    let high_thres = 0.6;
    let low_thres = 0.25;

    let mut queue = VecDeque::with_capacity(40);
    let mut thresholded: Vec<CannyLabel> = normalized.iter().enumerate().map(|(i, &f)| if f >= high_thres {queue.push_back(i); CannyLabel::Strong} else if f >= low_thres {CannyLabel::Weak} else {CannyLabel::Suppressed}).collect();

    while let Some(i) = queue.pop_front() {
        let x = (i % image.w) as i32;
        let y = (i / image.w) as i32;

        let mut try_queue_neigh = |a_x: i32, a_y: i32| {
            let (t_x, t_y) = (x + a_x, y + a_y);
            if t_x < 0 || t_x >= image.w as i32 || t_y < 0 || t_y >= image.h as i32 { return }
            let t_i = t_y as usize * image.w + t_x as usize;
            if thresholded[t_i] == CannyLabel::Weak {
                thresholded[t_i] = CannyLabel::Strong;
                queue.push_back(t_i);
            }
        };
        try_queue_neigh(-1,-1);
        try_queue_neigh(-1, 1);
        try_queue_neigh(-1, 0);
        try_queue_neigh(0, -1);
        try_queue_neigh(0,  1);
        try_queue_neigh(1, -1);
        try_queue_neigh(1,  0);
        try_queue_neigh(1,  1);
    }

    let final_res = thresholded.iter().map(|label| if (*label) == CannyLabel::Strong { true } else { false }).collect();

    Image {
        w: image.w,
        h: image.h,
        data: final_res,
    }
}

const TILE_WIDTH: f32 = 66.0;
const TILE_HEIGHT: f32 = 57.0;

fn get_font() -> Vec<(Marble, Image<bool>)> {
    let path = |marble: Marble| -> String { format!("symbol-font/{}.png", marble.to_string()) };
    (&MARBLE_VALUES[..]).iter().map(|&marble| {
        let im = image::open(path(marble));
        let image = match im {
            Ok(image) => image,
            Err(e) => panic!("Couldn't find sample image for {}, {}", marble.to_string(), e.description())
        };
        let luma = image.to_luma();
        let pixels = luma.into_raw().iter().map(|&l| l > 100).collect();
        (marble, Image{w: 30, h: 40, data: pixels})
    }).collect()
}

lazy_static! {
    static ref FONT: Vec<(Marble, Image<bool>)> = {
        get_font()
    };
}

fn matching_pixels(a: &Image<bool>, b: &Image<bool>) -> i32 {
    a.data.iter().enumerate().map(|(i, &k)| if k && b[i] { 1i32 } else { 0i32 }).sum()
}

fn recognize_marble_at(desktop_image: &Image<SRGB>, x: i32, y: i32) -> Marble {
    let mut data: Vec<SRGB> = Vec::with_capacity(30*40);
    for my in -20..20 {
        for mx in -15..15 {
            data.push(desktop_image[(x + mx) as usize + (y+my) as usize * desktop_image.w]);
        }
    }
    let gray_marble = Image{w: 30, h: 40, data: data.iter().map(|p| to_grayscale(*p)).collect()};

    let gauss = gray_marble.convolute(&GAUSS);

    let sobel_image = sobel(&gauss);
    let canny_image = canny(&sobel_image);

    let mut best_match = Marble::Empty;
    let mut best_match_count = 0;

    for &(sample_marble, ref sample_image) in FONT.iter() {
        let matching = matching_pixels(&canny_image, sample_image);
        if matching > best_match_count {
            best_match_count = matching;
            best_match = sample_marble;
        }
    }

    best_match
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

pub fn ocr_game_board() -> Option<Board>{
    let display = Display::primary().expect("Couldn't find primary display.");
    let capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (screen_w, screen_h) = (capturer.width(), capturer.height());

    let buffer = capture(capturer);

    // _save_screenshot(&buffer, screen_w, screen_h);

    let desktop_image: Image<SRGB> = Image{
        w: screen_w, h: screen_h,
        data: (&buffer[..]).chunks(4).map(|chunk: &[u8]| {
                match chunk {
                    &[b, g, r, _a] => SRGB{r, g, b},
                    _ => unreachable!()
                }
            }).collect()
    };

    let mut best_dist = ::std::f32::MAX;
    let mut best_coord = (0,0);

    for y in 10..(screen_h - 10) {
        for x in 10..(screen_w - 10) {
            let mut d: f32 = 0.0;
            for my in 0..3 {
                for mx in 0..5 {
                    let index = (y + my - 1) * screen_w + (x + mx - 2);
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

    // println!("Best guess for board center {} {:?}", best_dist, best_coord);
    let (gold_x, gold_y) = best_coord;

    let rows = board_rows();

    let mut board: Board = Board::new(
        [[Marble::Empty;13];13],
        gold_x as f32 / screen_w as f32,
        gold_y as f32 / screen_h as f32,
        TILE_WIDTH as f32 / screen_w as f32,
        TILE_HEIGHT as f32 / screen_h as f32,
    );

    for (i, r) in rows.iter().enumerate() {
        for x in r.x_min .. r.x_max + 1 {
            let (screen_x, screen_y) = board_pos_to_screen(x - 5, (i as i32 - 5));
            let (coord_x, coord_y) = (gold_x + (screen_x * TILE_WIDTH) as i32 + 1, gold_y + (screen_y * TILE_HEIGHT) as i32);
            
            board.board[i + 1][x as usize + 1] = recognize_marble_at(&desktop_image, coord_x, coord_y);
            // println!("{} {} {}", i, x, board.board[i + 1][x as usize + 1]);
        }
    }

    Some(board)
}

fn _save_screenshot(buffer: &Vec<u8>, buffer_w: usize, buffer_h: usize) {
    let mut bitflipped = Vec::with_capacity(buffer_w * buffer_h * 4);
    for pixel in buffer.chunks(4) {
        let (b, g, r, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        bitflipped.extend_from_slice(&[r, g, b, a]);
    }

    let path = Path::new("screenshot.png");
    let image: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(
            buffer_w as u32,
            buffer_h as u32,
            bitflipped
        ).expect("Couldn't convert frame into image buffer.");

    image.save(&path).expect("Couldn't save image to `screenshot.png`.");
    println!("Image saved to `screenshot.png`.");
}

fn board_pos_to_screen(x: i32, y: i32) -> (f32, f32) {
    (x as f32 + (y as f32 / 2.0), -y as f32)
}
