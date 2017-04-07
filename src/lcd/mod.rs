#![allow(dead_code)]

mod init;
mod color;
mod primitives;

pub use self::color::Color;
pub use self::init::init;
pub use self::primitives::*;

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

const LCD_SIZE: Rect = Rect{
    origin: Point{x:0,y:0},
    width: 480,
    height: 272,
};

const CLEAR_COLOR: u16 = 0;

impl Lcd {

    #[inline]
    pub fn point_addr(&mut self, point: Point, layer: Layer) -> *mut u16 {
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
        self.fill_rect_color(LCD_SIZE, Layer1, CLEAR_COLOR);
        self.fill_rect_color(LCD_SIZE, Layer2, CLEAR_COLOR);
    }

    #[inline]
    pub fn draw_point_color(&mut self, p: Point, l: Layer, c: u16) {
        let addr = self.point_addr(p, l);
        unsafe { ptr::write_volatile(addr, c) };
    }

    pub fn fill_rect_color(&mut self, r: Rect, l: Layer, c: u16) {
        r.foreach_point(|point| self.draw_point_color(point, l, c));
    }

}
