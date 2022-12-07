mod segment;
mod tree;

use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use clap::Parser;
use kurbo::{ParamCurve, ParamCurveExtrema, PathSeg, Point, Rect, Size};
use svg::node::element::{tag, Definitions, Marker, Path};
use svg::parser::Event;
use svg::Document;

use segment::AbstractSegment;
use tree::TreeNode;

#[derive(Parser)]
struct Settings {
    #[arg()]
    input: PathBuf,
}

fn main() {
    let settings = Settings::parse();

    let mut document_attrs = HashMap::new();
    let mut segments = vec![];

    let mut content = String::new();
    for event in svg::open(&settings.input, &mut content).unwrap() {
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
    for seg in &segments {
        let mut seg = seg.clone();
        let data = seg.remove("d").unwrap().to_string();
        let path = kurbo::BezPath::from_svg(&data).unwrap();
        let mut id = 0;
        for segment in path.segments() {
            match segment {
                p @ PathSeg::Line(_) => {
                    abstract_segments.push(AbstractSegment {
                        path: p,
                        attributes: seg.clone(),
                        id,
                    });
                    id += 1;
                }
                PathSeg::Quad(quad_bez) => {
                    for range in quad_bez.extrema_ranges() {
                        let subseg = quad_bez.subsegment(range);
                        abstract_segments.push(AbstractSegment {
                            path: PathSeg::Quad(subseg),
                            attributes: seg.clone(),
                            id,
                        });
                        id += 1;
                    }
                }
                PathSeg::Cubic(cubic_bez) => {
                    let mut cuts = vec![0.0, 1.0];
                    cuts.extend(cubic_bez.extrema());
                    cuts.extend(cubic_bez.inflections());
                    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    cuts.dedup_by(|&mut a, &mut b| (a - b).abs() < 0.01);

                    for range in cuts.windows(2) {
                        let &[start, end] = range else { panic!("Windows did not return a slice of size 2" )};
                        let range = Range { start, end };
                        let cubic_seg = cubic_bez.subsegment(range.clone());
                        abstract_segments.push(AbstractSegment {
                            path: PathSeg::Cubic(cubic_seg),
                            attributes: seg.clone(),
                            id,
                        });
                        id += 1;
                    }
                }
            }
        }
    }

    let view_box: Vec<f64> = document_attrs
        .get("viewBox")
        .unwrap()
        .split(' ')
        .map(|s| s.parse().unwrap())
        .collect();
    let bounding_box = Rect::from_origin_size(
        Point::new(view_box[0], view_box[1]),
        Size::new(view_box[2], view_box[3]),
    );
    let tree = TreeNode::new(bounding_box, abstract_segments.clone(), 0);

    let groups = tree.output();

    // Set up the SVG document
    let mut doc = Document::new();
    for (k, v) in document_attrs {
        doc = doc.set(k, v);
    }

    // Add the marker
    doc = doc.add(
        Definitions::new().add(
            Marker::new()
                .set("id", "arrow")
                .set("viewBox", (0, 0, 10, 10))
                .set("refX", 5)
                .set("refY", 5)
                .set("markerWidth", 6)
                .set("markerHeight", 6)
                .set("orient", "auto-start-reverse")
                .add(
                    Path::new()
                        .set("d", "M 0 0 L 10 5 L 0 10 z")
                        .set("fill", "black")
                        .set("stroke", "black"),
                ),
        ),
    );

    for segment in segments {
        let mut p = Path::new();
        for (k, v) in segment {
            p = p.set(k, v);
        }
        doc = doc.add(p.set("class", "original"))
    }

    // Add the tree nodes
    for group in groups {
        doc = doc.add(group);
    }

    // Add segment bounding boxes
    for segment in &abstract_segments {
        doc = doc.add(segment.bounding_box());
    }

    // Add the segments
    for segment in &abstract_segments {
        doc = doc.add(segment.to_svg_group());
    }
    
    let mut output = PathBuf::from("web/public");
    output.push(settings.input);
    svg::save(&output, &doc).unwrap();
}
