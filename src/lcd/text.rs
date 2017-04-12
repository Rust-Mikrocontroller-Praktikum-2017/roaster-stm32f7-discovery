extern crate stb_truetype;
extern crate font_rs;

//use self::stb_truetype;
use self::stb_truetype::{FontInfo};
use self::font_rs::font;

use super::primitives::*;
use super::color::Color;

use core::cmp::max;
use core::result::Result;
use core::fmt::Write;
use core;

pub struct Font<'a> {
    font_info: FontInfo<&'a [u8]>,
    font: font::Font<'a>,
    pub size: u16,
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

/// Public interface
pub struct TextBox<'a> {
    pub alignment: Alignment,
    pub canvas: Rect,
    pub font: &'a Font<'a>,
    pub bg_color: Color,
    pub fg_color: Color,
}


#[derive(Copy,Clone)]
pub enum Alignment {
    Left,
    Center,
    Right
}

/// Internal Writer used to do the per-point drawing calls to the callback
///
/// Relationship between TextBox.canvas (TB canvas) and
///     TextWriter.canvas (TW canvas):

///   -----------------------
///   |[TB canvas]           |  <-- alignment determines position of TB canvas
///   -----------------------
///   <--[TW canvas width]-->
struct TextWriter<'a,F: FnMut(Point,Color)> {
    /// The
    canvas: Rect,
    font: &'a Font<'a>,
    bg_color: Color,
    fg_color: Color,

    draw: F,
    off: Point,
}

impl<'a> TextBox<'a> {

    /// Draw string s to the canvas
    ///
    /// F is a closure that should draw to the output
    /// - Point: the absolute coordinate (Point is in self.canvas)
    /// - Color: the color (self.fg_color)
    ///
    /// Internally, TextWriter is used twice to
    ///   1. determine the effectively required canvas size
    ///   2. do the actual drawing
    pub fn redraw<F>(&'a self, s: &str, mut draw: F)
        where F: FnMut(Point,Color) {

        // First run: capture max bounds, but don't draw
        let mut effective_canvas = self.canvas;
        {
            effective_canvas.width = 0;
            effective_canvas.height = 0;

            let mut w = TextWriter{
                font: self.font,
                bg_color: self.bg_color,
                fg_color: self.fg_color,
                canvas: self.canvas,
                off: self.canvas.origin,
                draw: |p,_| effective_canvas.extend_to_point(p),
            };
            write!(&mut w, "{}", s).unwrap();
        }

        // Clear background
        effective_canvas.foreach_point(|p| draw(p, self.bg_color));

        // Determine alignment offset
        let alignment_offset = match self.alignment {
            Alignment::Left   => Point{x:0,y:0},
            Alignment::Center => Point{x: (self.canvas.width - effective_canvas.width)/2,
                                       y: 0},
            Alignment::Right  => Point{x:  self.canvas.width - effective_canvas.width,
                                       y: 0},
        };

        let mut aligned_canvas = effective_canvas;
        aligned_canvas.origin += alignment_offset;

        // Second pass, call user-provided drawing callback this time
        let mut w = TextWriter{
            font: self.font,
            bg_color: self.bg_color,
            fg_color: self.fg_color,
            canvas: aligned_canvas,
            off: aligned_canvas.origin,
            draw: draw,
        };
        write!(&mut w, "{}", s).unwrap();

    }
}

impl<'a, F: FnMut(Point,Color)> Write for TextWriter<'a, F>{

    fn write_str(&mut self, s: &str) -> Result<(),core::fmt::Error> {

        for c in s.chars() {

            let mut c = c;

            let char_needs_render = match c {
                '\n' => {
                    self.off = Point{x: self.canvas.origin.x,
                                y: self.off.y + self.font.size};
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

            let advance_width = {
                let scale = self.font.font_info.scale_for_mapping_em_to_pixels(self.font.size as f32);
                let unscaled = self.font.font_info.get_glyph_h_metrics(glyph_id).advance_width;
                scale * (unscaled as f32)
            };

            // Line Wrapping
            let mut new_x_off = self.off.x + (advance_width as i32) as u16 ;
            if char_needs_render &&
               new_x_off > self.canvas.anchor_point(Anchor::UpperRight).x{

                self.off = Point{ x: self.canvas.origin.x, y: self.off.y + self.font.size};
                new_x_off = self.canvas.origin.x + (advance_width as i32) as u16;

            }

            // Height limit
            let max_char_bottom = self.off.y + self.font.size;
            if max_char_bottom > self.canvas.anchor_point(Anchor::LowerRight).y {
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
                        p.x = max(((self.off.x + x) as i32), 0) as u16;
                        let fs = self.font.size as i32;
                        p.y = max(((self.off.y + y) as i32) + glyph.top + fs, 0) as u16;

                        let c = mix_color(self.fg_color, self.bg_color, shade);

                        (self.draw)(p, c);
                    }
                }
                for y in 0..glyph.height {
                    for x in glyph.width..(advance_width as usize) {
                        let p = Point{
                            x: self.off.x + (x as u16),
                            y: self.off.y + (y as u16),
                        };
                        (self.draw)(p, self.bg_color);
                    }
                }
            }

            self.off.x = new_x_off;

        }

        Ok(())

    }
}

fn mix_color(a: Color, b: Color, a_amt: u8) -> Color {
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