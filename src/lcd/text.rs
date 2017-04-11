extern crate stb_truetype;
extern crate font_rs;

//use self::stb_truetype;
use self::stb_truetype::{FontInfo};
use self::font_rs::font;

use super::primitives::*;
use super::color::Color;

use core::cmp::max;
use core::result::Result;

pub struct Font<'a> {
    font_info: FontInfo<&'a [u8]>,
    font: font::Font<'a>,
    size: u16,
}

#[derive(Debug)]
pub enum Error {
    FontInfo,
    Parse
}

impl<'a> Font<'a> {
    pub fn new(ttf: &'a [u8], size: u16) -> Result<Font,Error> {
        Ok(Font {
            font_info: FontInfo::new(ttf, 0).ok_or(Error::FontInfo)?,
            font: font::parse(ttf).or(Err(Error::Parse))?,
            size: size,
        })
    }
}

pub struct TextBox<'a> {
    pub canvas: Rect,
    pub font: &'a Font<'a>,
    pub text: &'static str,
    pub alignment: Alignment,
    pub bg_color: Color,
    pub fg_color: Color,
}

#[derive(Copy,Clone)]
pub enum Alignment {
    Left,
}

impl<'a> TextBox<'a> {

    /// F is a closure that should draw to the output
    /// - Point: the absolute coordinate (Point is in self.canvas)
    /// - Color: the color (self.fg_color)
    pub fn redraw<F>(&self, mut draw: F)
        where F: FnMut(Point,&Color) {

        // Clear background
        self.canvas.foreach_point(|p| draw(p, &self.bg_color));

        let mut off = self.canvas.origin;

        for c in self.text.chars() {

            let mut c = c;

            let char_needs_render = match c {
                '\n' => {
                    off = Point{x: self.canvas.origin.x,
                                y: off.y + self.font.size};
                    continue
                },
                ' ' => {
                    c = '-';
                    false
                },
                _   => true
            };

            let glyph_id = self.font.font_info.find_glyph_index(c.into());
            let glyph = self.font.font
                .render_glyph(glyph_id as u16, self.font.size as u32)
                .expect("Failed to render glyph");

            // Line Wrapping
            let mut new_x_off = off.x + (glyph.width as i32 + glyph.left) as u16 ;
            if char_needs_render &&
               new_x_off >= self.canvas.anchor_point(Anchor::UpperRight).x {

                off = Point{ x: self.canvas.origin.x, y: off.y + self.font.size};
                new_x_off = self.canvas.origin.x + (glyph.width as i32 + glyph.left) as u16;

            }

            // Height limit
            let max_char_bottom = off.y + self.font.size;
            if max_char_bottom >= self.canvas.anchor_point(Anchor::LowerRight).y {
                break;
            }

            // Render char with appropriate offsets
            if char_needs_render {
                for y in 0..glyph.height {
                    for x in 0..glyph.width {

                        let shade = glyph.data[y * glyph.width + x];

                        let x = x as u16;
                        let y = y as u16;
                        let mut p = Point{x: 0, y: 0};
                        p.x = max(((off.x + x) as i32) + glyph.left, 0) as u16;
                        let fs = self.font.size as i32;
                        p.y = max(((off.y + y) as i32) + glyph.top + fs, 0) as u16;

                        let c = mix_color(&self.fg_color, &self.bg_color, shade);

                        draw(p, &c);
                    }
                }
            }

            off.x = new_x_off;

        }

    }
}

fn mix_color(a: &Color, b: &Color, a_amt: u8) -> Color {
    Color{
        red: mix_u8(a.red, b.red, a_amt),
        green: mix_u8(a.green, b.green, a_amt),
        blue: mix_u8(a.blue, b.blue, a_amt),
        alpha: mix_u8(a.alpha, b.alpha, a_amt),
    }
}

fn mix_u8(a: u8, b: u8, a_amt: u8) -> u8 {
    let a = a as usize;
    let b = b as usize;

    let full = 255 * 1000;
    let a_amt  = a_amt as usize * 1000;
    let b_amt = full - a_amt;

    let res = (a * a_amt + b * b_amt) / full;
    return res as u8;
}