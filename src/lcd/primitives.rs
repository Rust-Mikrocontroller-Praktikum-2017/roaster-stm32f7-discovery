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
    LowerLeft,
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
            Anchor::LowerLeft => Point{x: self.origin.x, y: self.origin.y + self.height},
        }
    }

    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        let Point{x,y} = *p;
        let Point{x:ox,y:oy} = self.origin;
        return (x >= ox && x <= ox + self.width)
            && (y >= oy && y <= oy + self.height);
    }

    pub fn union(a: Rect, b: Rect) -> Rect {
        let a_max = a.anchor_point(Anchor::LowerRight);
        let b_max = b.anchor_point(Anchor::LowerRight);

        let o = Point::min(a.origin, b.origin);
        let max = Point::max(a_max, b_max);

        Rect{
            origin: o,
            width: max.x - o.x,
            height: max.y - o.y,
        }
    }

}

impl Point {
    fn max(a: Point, b: Point) -> Point {
        use core::cmp::max;
        Point{
            x: max(a.x,b.x),
            y: max(a.y,b.y),
        }
    }
    fn min(a: Point, b: Point) -> Point {
        use core::cmp::min;
        Point{
            x: min(a.x, b.x),
            y: min(a.y, b.y),
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
