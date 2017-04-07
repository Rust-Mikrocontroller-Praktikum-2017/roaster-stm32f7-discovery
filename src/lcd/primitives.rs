pub struct Point {
    pub x: u16,
    pub y: u16,
}

pub struct Line {
    pub from: Point,
    pub to: Point,
}

pub struct Rect {
    pub  origin: Point,
    pub  width: u16,
    pub  height: u16,
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
}



