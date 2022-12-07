use std::collections::HashMap;

use kurbo::{CubicBez, Line, ParamCurve, PathSeg, Point, QuadBez, Shape};
use svg::node::{
    element::{Circle, Group, Path, Rectangle},
    Value,
};

#[derive(Debug, Clone)]
pub struct AbstractSegment {
    pub id: usize,
    pub path: PathSeg,
    pub attributes: HashMap<String, Value>,
}

impl AbstractSegment {
    pub fn to_svg_path(&self) -> Group {
        let (seg1, seg2) = self.split_path();

        let mut path1 = Path::new();
        let mut path2 = Path::new();

        for (k, v) in &self.attributes {
            path1 = path1.set(k, v.clone());
            path2 = path2.set(k, v.clone());
        }

        path1 = path1.set("d", seg1.to_path(0.1).to_svg());
        path2 = path2.set("d", seg2.to_path(0.1).to_svg());
        
        path1 = path1.set("class", "firstHalf");

        Group::new().add(path1).add(path2)
    }

    pub fn to_svg_group(&self) -> Group {
        let (start, end) = self.get_points();
        Group::new()
            .set("id", format!("segment{}", self.id))
            .set("class", "segment")
            .add(self.to_svg_path())
            .add(
                Circle::new()
                    .set("cx", start.x)
                    .set("cy", start.y)
                    .set("r", 2)
                    .set("fill", "black")
                    .set("class", "segmentBoundary"),
            )
            .add(
                Circle::new()
                    .set("cx", end.x)
                    .set("cy", end.y)
                    .set("r", 2)
                    .set("fill", "black")
                    .set("class", "segmentBoundary"),
            )
    }

    fn split_path(&self) -> (PathSeg, PathSeg) {
        match self.path {
            PathSeg::Line(line) => {
                let (a, b) = line.subdivide();
                (PathSeg::Line(a), PathSeg::Line(b))
            }
            PathSeg::Quad(quad) => {
                let (a, b) = quad.subdivide();
                (PathSeg::Quad(a), PathSeg::Quad(b))
            }
            PathSeg::Cubic(cube) => {
                let (a, b) = cube.subdivide();
                (PathSeg::Cubic(a), PathSeg::Cubic(b))
            }
        }
    }

    pub fn get_points(&self) -> (Point, Point) {
        match self.path {
            PathSeg::Line(Line { p0, p1 }) => (p0, p1),
            PathSeg::Quad(QuadBez { p0, p2, .. }) => (p0, p2),
            PathSeg::Cubic(CubicBez { p0, p3, .. }) => (p0, p3),
        }
    }

    #[allow(unused)]
    pub fn bounding_box(&self) -> Rectangle {
        let bbox = Shape::bounding_box(&self.path);
        Rectangle::new()
            .set("class", "segment_bounding_box")
            .set("x", bbox.x0)
            .set("y", bbox.y0)
            .set("width", bbox.x1 - bbox.x0)
            .set("height", bbox.y1 - bbox.y0)
            .set("stroke", "blue")
            .set("fill", "none")
            .set("stroke-dasharray", "4")
    }
}
