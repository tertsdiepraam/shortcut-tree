use std::collections::HashMap;
use std::ops::Range;

use kurbo::{ParamCurve, ParamCurveExtrema, PathSeg, Shape};
use svg::node::element::{tag, Path, Rectangle};
use svg::node::Value;
use svg::parser::Event;
use svg::Document;

#[derive(Debug)]
struct AbstractSegment {
    path: PathSeg,
    attributes: HashMap<String, Value>,
}

impl AbstractSegment {
    fn to_svg_path(&self) -> Path {
        let mut path = Path::new();
        for (k, v) in &self.attributes {
            path = path.set(k, v.clone());
        }
        path.set("d", self.path.to_path(0.1).to_svg())
    }

    fn bounding_box(&self) -> Rectangle {
        let bbox = Shape::bounding_box(&self.path);
        Rectangle::new()
            .set("x", bbox.x0)
            .set("x", bbox.x0)
            .set("y", bbox.y0)
            .set("width", bbox.x1 - bbox.x0)
            .set("height", bbox.y1 - bbox.y0)
            .set("stroke", "blue")
            .set("fill", "none")
    }
}

fn main() {
    let path = "image.svg";
    let mut document_attrs = HashMap::new();

    let mut content = String::new();

    let mut segments = vec![];

    for event in svg::open(path, &mut content).unwrap() {
        match event {
            Event::Tag(tag::Path, _, attributes) => {
                segments.push(attributes);
            }
            Event::Tag(tag::SVG, tag::Type::Start, attributes) => {
                document_attrs.extend(attributes);
            }
            _ => {}
        }
    }

    // Create abstract segments from segments:
    //  - Lines are kept as lines
    //  - Quadratic bezier curves are split at extrema
    //  - Cubic bezier curves are split at extrema and inflection points
    let mut abstract_segments: Vec<AbstractSegment> = Vec::new();
    for mut seg in segments {
        let data = seg.remove("d").unwrap().to_string();
        let path = kurbo::BezPath::from_svg(&data).unwrap();
        for segment in path.segments() {
            match segment {
                p @ PathSeg::Line(_) => abstract_segments.push(AbstractSegment {
                    path: p,
                    attributes: seg.clone(),
                }),
                PathSeg::Quad(quad_bez) => {
                    for range in quad_bez.extrema_ranges() {
                        let subseg = quad_bez.subsegment(range);
                        abstract_segments.push(AbstractSegment {
                            path: PathSeg::Quad(subseg),
                            attributes: seg.clone(),
                        })
                    }
                }
                PathSeg::Cubic(cubic_bez) => {
                    for range in cubic_bez.extrema_ranges() {
                        let cubic_seg = cubic_bez.subsegment(range);

                        // Get inflections including 0 and 1 so we can construct ranges
                        // based on those values.
                        let mut inflections = vec![0.0];
                        inflections.extend(cubic_seg.inflections());
                        inflections.push(1.0);

                        for range in inflections.windows(2) {
                            let &[start, end] = range else { panic!("Windows did not return a slice of size 2" )};
                            let range = Range { start, end };
                            let subseg = cubic_seg.subsegment(range);
                            abstract_segments.push(AbstractSegment {
                                path: PathSeg::Cubic(subseg),
                                attributes: seg.clone(),
                            })
                        }
                    }
                }
            }
        }
    }

    // Set up the document
    let mut doc = Document::new();
    for (k, v) in document_attrs {
        doc = doc.set(k, v);
    }

    // Add the abstract segments
    for segment in &abstract_segments {
        doc = doc.add(segment.to_svg_path());
    }

    // Add the bounding boxes of the segments
    for segment in &abstract_segments {
        doc = doc.add(segment.bounding_box());
    }

    svg::save("output.svg", &doc).unwrap();
}
