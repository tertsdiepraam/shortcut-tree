use kurbo::{Line, ParamCurve, Point, Rect, Shape, Size};
use svg::node::{
    element::{Group, Path, Rectangle, Text},
    Text as TextNode,
};

use crate::segment::AbstractSegment;

#[derive(Debug)]
enum NodeType {
    Leaf(Vec<AbstractSegment>),
    Branch(Box<TreeNode>, Box<TreeNode>, Box<TreeNode>, Box<TreeNode>),
}

#[derive(Debug)]
pub struct TreeNode {
    bounding_box: Rect,
    node_type: NodeType,
    initial_winding_number: i64,
}

const MAX_DEPTH: usize = 4;

impl TreeNode {
    pub fn new(
        bounding_box: Rect,
        segments: Vec<AbstractSegment>,
        initial_winding_number: i64,
    ) -> Self {
        TreeNode {
            bounding_box,
            node_type: NodeType::Leaf(segments),
            initial_winding_number,
        }
        .split(0)
    }

    fn count_bottom_intersections(&self) -> i64 {
        let NodeType::Leaf(segments) = &self.node_type else {
            panic!("Called count_bottom_intersections on a branch instead of leaf");
        };

        let bottom_left = Point::new(self.bounding_box.x0, self.bounding_box.y1);
        let bottom_right = Point::new(self.bounding_box.x1, self.bounding_box.y1);
        let bottom_line = Line::new(bottom_left, bottom_right);

        let mut intersections = 0;

        for segment in segments {
            if !segment.path.intersect_line(bottom_line).is_empty() {
                if segment.get_points().0.y < bottom_left.y {
                    intersections -= 1;
                } else {
                    intersections += 1;
                }
            }
        }

        intersections
    }

    fn split(self, depth: usize) -> Self {
        let NodeType::Leaf(segments) = &self.node_type else {
            panic!("Called split on node that is already branched!");
        };

        // Stop condition: too few segments
        if segments.len() <= 2 {
            return self;
        }

        if depth >= MAX_DEPTH {
            return self;
        }

        // Split self
        let origin = self.bounding_box.origin();
        let center = self.bounding_box.center();
        let size = self.bounding_box.size() / 2.0;

        // origin
        // v
        // *------*------*
        // |      |      |
        // |      |v-----|---center
        // *------*------*
        // |      |      |
        // |      |      |
        // *------*------*
        let new_origins = (
            // Top-left rectangle
            origin,
            // Top-right rectangle
            Point::new(center.x, origin.y),
            // Bottom-left rectangle
            Point::new(origin.x, center.y),
            // Bottom-right rectangle
            center,
        );

        let mut top_left = TreeNode::filter_segments(segments, new_origins.0, size);
        let mut top_right = TreeNode::filter_segments(segments, new_origins.1, size);
        let mut bottom_left = TreeNode::filter_segments(segments, new_origins.2, size);
        let mut bottom_right = TreeNode::filter_segments(segments, new_origins.3, size);

        // Fix up the winding numbers
        bottom_left.initial_winding_number = self.initial_winding_number;
        bottom_right.initial_winding_number = self.initial_winding_number;
        
        let halfway = (self.bounding_box.y0 + self.bounding_box.y1) / 2.0;
        let middle_right = Point::new(self.bounding_box.x1, halfway);
        let middle_inf = Point::new(1000.0, halfway);
        let section_f = Line::new(middle_right, middle_inf);

        let right_edge = Line::new(Point::new(self.bounding_box.x1, self.bounding_box.y0), Point::new(self.bounding_box.x1, self.bounding_box.y1));

        let mut section_f_intersections = 0;
        for segment in segments {
            let (start, end) = segment.get_points();
            if !segment.path.intersect_line(section_f).is_empty() {
                if start.y < halfway {
                    section_f_intersections -= 1;
                } else {
                    section_f_intersections += 1;
                }
            }

            // Add intersections for abstract segments
            if !segment.path.intersect_line(right_edge).is_empty() {
                if start.x > self.bounding_box.x1 && start.y > halfway {
                    section_f_intersections -= 1;
                }
                if end.x > self.bounding_box.x1 && end.y > halfway {
                    section_f_intersections += 1;
                }
            }
        }

        top_right.initial_winding_number = self.initial_winding_number + section_f_intersections;
        top_left.initial_winding_number = self.initial_winding_number +
            top_right.count_bottom_intersections() + section_f_intersections;
        bottom_left.initial_winding_number =
            self.initial_winding_number + bottom_right.count_bottom_intersections();
        bottom_right.initial_winding_number = self.initial_winding_number;

        Self {
            bounding_box: self.bounding_box,
            node_type: NodeType::Branch(
                Box::new(top_left.split(depth + 1)),
                Box::new(top_right.split(depth + 1)),
                Box::new(bottom_left.split(depth + 1)),
                Box::new(bottom_right.split(depth + 1)),
            ),
            initial_winding_number: self.initial_winding_number,
        }
    }

    fn filter_segments(segments: &[AbstractSegment], origin: Point, size: Size) -> Self {
        let bounding_box = Rect::from_origin_size(origin, size);

        let top_left = Point::new(bounding_box.x0, bounding_box.y0);
        let top_right = Point::new(bounding_box.x1, bounding_box.y0);
        let bottom_left = Point::new(bounding_box.x0, bounding_box.y1);
        let bottom_right = Point::new(bounding_box.x1, bounding_box.y1);
        let bounding_lines = [
            Line::new(top_left, top_right),
            Line::new(top_right, bottom_right),
            Line::new(bottom_left, bottom_right),
            Line::new(top_left, bottom_left),
        ];

        let my_segments = segments
            .iter()
            .filter(|seg| {
                let (start, end) = seg.get_points();
                bounding_box.contains(start)
                    || bounding_box.contains(end)
                    || bounding_lines
                        .iter()
                        .any(|&l| !seg.path.intersect_line(l).is_empty())
            })
            .cloned()
            .collect();

        Self {
            bounding_box,
            node_type: NodeType::Leaf(my_segments),
            initial_winding_number: 0,
        }
    }

    pub fn output(&self) -> Vec<Group> {
        match &self.node_type {
            NodeType::Branch(a, b, c, d) => {
                let mut groups = vec![];
                groups.extend(a.output());
                groups.extend(b.output());
                groups.extend(c.output());
                groups.extend(d.output());
                groups
            }
            NodeType::Leaf(segments) => {
                let top_right = Point::new(self.bounding_box.x1, self.bounding_box.y0);
                let bottom_right = Point::new(self.bounding_box.x1, self.bounding_box.y1);

                let mut shortcuts = Vec::new();

                let line = Line::new(top_right, bottom_right);
                for segment in segments {
                    if !segment.path.intersect_line(line).is_empty() {
                        let (start, end) = segment.get_points();
                        if start.x > end.x {
                            // create incoming shortcut segment
                            shortcuts.push(Line::new(
                                Point::new(start.x, self.bounding_box.y0 - 20.0),
                                start,
                            ))
                        } else {
                            // create outgoing shortcut segment
                            shortcuts.push(Line::new(
                                end,
                                Point::new(end.x, self.bounding_box.y0 - 20.0),
                            ))
                        }
                    }
                }

                let shortcut_paths: Vec<_> = shortcuts
                    .iter()
                    .map(|line| {
                        let (a, b) = line.subdivide();
                        [
                            Path::new()
                                .set("d", a.to_path(0.1).to_svg())
                                .set("class", "shortcut firstHalf arrowhead"),
                            Path::new()
                                .set("d", b.to_path(0.1).to_svg())
                                .set("class", "shortcut"),
                        ]
                    })
                    .flatten()
                    .map(|p| p.set("stroke", "black").set("stroke-dasharray", "2"))
                    .collect();

                let mut group = Group::new()
                    .set("class", "treenode")
                    .set(
                        "segments",
                        segments
                            .iter()
                            .map(|s| format!("segment{}", s.id))
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                    .add(
                        Rectangle::new()
                            .set("x", self.bounding_box.x0)
                            .set("y", self.bounding_box.y0)
                            .set("width", self.bounding_box.width())
                            .set("height", self.bounding_box.height())
                            .set("stroke", "#aaa")
                            .set("fill", "transparent"),
                    )
                    .add(
                        Text::new()
                            .set("x", self.bounding_box.x0 + 1.0)
                            .set("y", self.bounding_box.y0 + 6.0)
                            .set("class", "winding_number")
                            .add(TextNode::new(format!("{}", self.initial_winding_number))),
                    );

                for shortcut in shortcut_paths {
                    group = group.add(shortcut);
                }

                vec![group]
            }
        }
    }
}
