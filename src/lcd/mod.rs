#![allow(dead_code)]

mod init;
mod color;
mod primitives;
mod text;

pub use self::color::Color;
pub use self::init::init;
pub use self::primitives::*;
pub use self::text::*;

use board::ltdc::Ltdc;
use embedded::interfaces::gpio::OutputPin;
use core::ptr;

pub struct Lcd {
    controller: &'static mut Ltdc,
    display_enable: OutputPin,
    backlight_enable: OutputPin,
    prev_value: (u32, u32),
}

#[derive(Copy,Clone)]
pub enum Layer {
    Layer1,
    Layer2,
}
use self::Layer::*;

pub const LCD_SIZE: Rect = Rect{
    origin: Point{x:0,y:0},
    width: 480,
    height: 272,
};

pub const CLEAR_COLOR: Color = Color{red: 0, green: 0, blue: 0, alpha: 0};

impl Lcd {

    #[inline]
    pub fn point_addr(&mut self, point: Point, layer: Layer) -> *mut u16 {
        assert!(LCD_SIZE.contains_point(&point));
        let base: u32 = match layer {
            Layer1  =>  0xC000_0000,
            Layer2  =>  {
                let mut addr = LCD_SIZE.width as u32;
                addr *= LCD_SIZE.height as u32;
                addr *= 2;
                addr += 0xC000_0000;
                addr
            }
        };
        let mut pixel_offset: u32 = point.y as u32;
        pixel_offset *= LCD_SIZE.width as u32;
        pixel_offset += point.x as u32;
        pixel_offset *= 2;
        return (base + pixel_offset) as *mut u16;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.controller
            .bccr
            .update(|r| r.set_bc(color.to_rgb()));
    }

    pub fn clear_screen(&mut self) {
        self.fill_rect_color(LCD_SIZE, Layer1, CLEAR_COLOR.to_argb1555());
        self.fill_rect_color(LCD_SIZE, Layer2, CLEAR_COLOR.to_argb1555());
    }

    #[inline]
    pub fn draw_point_color(&mut self, p: Point, l: Layer, c: u16) {
        if !LCD_SIZE.contains_point(&p) {
            return;
        }
        let addr = self.point_addr(p, l);
        unsafe { ptr::write_volatile(addr, c) };
    }

    pub fn fill_rect_color(&mut self, r: Rect, l: Layer, c: u16) {
        r.foreach_point(|point| self.draw_point_color(point, l, c));
    }

    /// Render a line using the Bresenham algorithm
    pub fn draw_line_color(&mut self, line: Line, layer: Layer, color: u16) {
        let Line {
            from,
            to
        } = line;

        if !(from.x < LCD_SIZE.width && to.x < LCD_SIZE.width) || !(from.y < LCD_SIZE.height && to.y < LCD_SIZE.height){
            return;
        }

        let x0 = from.x as i16;
        let y0 = from.y as i16;
        let x1 = to.x as i16;
        let y1 = to.y as i16;

        let dx:i16 = (x0 - x1).abs();
        let sx:i16 = if from.x < to.x {1} else {-1};
        let dy:i16 = -(y0 - y1).abs();
        let sy:i16 = if from.y < to.y {1} else {-1};

        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_point_color(Point{x:x as u16, y:y as u16}, layer, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 > dy {
                err += dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

}
