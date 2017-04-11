#[derive(Copy, Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Copy, Clone)]
pub struct Line {
    pub from: Point,
    pub to: Point,
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub  origin: Point,
    pub  width: u16,
    pub  height: u16,
}

#[derive(PartialEq,Copy,Clone)]
pub enum Anchor {
    UpperRight,
    LowerRight,
}

impl Rect {
    pub fn foreach_point<V>(&self, mut v: V) where V: FnMut(Point) {
        for i in (self.origin.y)..(self.origin.y+self.height) {
            for j in (self.origin.x)..(self.origin.x + self.width){
                let p = Point{x: j, y: i};
                v(p);
            }
        }
    }

    #[inline]
    pub fn anchor_point(&self, anchor: Anchor) -> Point {
        match anchor {
            Anchor::UpperRight => Point{x: self.origin.x + self.width, y: self.origin.y},
            Anchor::LowerRight => Point{x: self.origin.x + self.width,
                                        y: self.origin.y + self.height},
        }
    }

    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        let Point{x,y} = *p;
        let Point{x:ox,y:oy} = self.origin;
        return (x >= ox && x <= ox + self.width)
            && (y >= oy && y <= oy + self.height);
    }

    pub fn extend_to_point(&mut self, p: Point) {
        let dx = p.x as i32 - (self.origin.x + self.width) as i32;
        let dy = p.y as i32 - (self.origin.y + self.height) as i32;
        if dx >= 0 {
            self.width += dx as u16 + 1;
        }
        if dy >= 0 {
            self.height += dy as u16 + 1;
        }
    }

}

use core::ops::{Add,AddAssign};

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        *self = Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}
