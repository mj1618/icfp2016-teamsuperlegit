use svg;
use svg::Document;
use svg::node::element;

use ::BASEPATH;
use core::*;

pub fn draw_svg<N: Num>(shape: Shape<N>, skel: Skeleton<N>, filename: &str) {
	/* Draw shapes as areas and skeletons as lines */
	let mut document = Document::new().set("viewBox", (0, 0, 1, 1));
    let mut defs = element::Definitions::new();
    let marker_path_start = element::Path::new()
            .set("fill", "crimson")
            .set("d",  "M 0.0,0.0 L 5.0,-5.0 L -12.5,0.0 L 5.0,5.0 L 0.0,0.0 z")
            .set("transform", "scale(0.2) rotate(180) translate(10,0)");
    let marker_path_end = element::Path::new()
            .set("fill", "crimson")
            .set("d",  "M 0.0,0.0 L 5.0,-5.0 L -12.5,0.0 L 5.0,5.0 L 0.0,0.0 z")
            .set("transform", "scale(0.2) rotate(180) translate(10,0)");
    let marker_start = element::Marker::new()
            .set("style", "overflow:visible")
            .set("orient", "auto")
            .set("id", "ArrowStart")
            .add(marker_path_start);
    let marker_end = element::Marker::new()
            .set("style", "overflow:visible")
            .set("orient", "auto")
            .set("id", "ArrowEnd")
            .add(marker_path_end);
    defs = defs.add(marker_start);
    defs = defs.add(marker_end);
    document = document.add(defs);

	for polygon in shape {
		let mut iter = polygon.points.iter();		
		let startpoint = iter.next().unwrap();
		let mut data = element::path::Data::new().move_to((startpoint.x.to_f64(), startpoint.y.to_f64()));
		println!("{:?}, {:?}", startpoint.x.to_f64(), startpoint.y.to_f64());
		// path.move_to(shape.points[0] (as float))
		for point in iter {
			println!("{:?}, {:?}", point.x.to_f64(), point.y.to_f64());
			data = data.line_to((point.x.to_f64(), point.y.to_f64()));
		}
        data = data.close();
		let path = element::Path::new()
				.set("fill", "none")
				.set("stroke", "black")
				.set("stroke-width", 0.02)
				.set("d", data);
		document = document.add(path);
	}
    for bone in skel {
        let skel_data = element::path::Data::new()
                .move_to((bone.p1.x.to_f64(), bone.p1.y.to_f64()))
                .line_to((bone.p2.x.to_f64(), bone.p2.y.to_f64()));
        let skel_path = element::Path::new()
                .set("fill", "none")
                .set("stroke", "crimson")
                .set("stroke-width", 0.015)
                .set("stroke-dasharray", "0.01,0.01")
                .set("marker-start", "url(#ArrowStart)")
                .set("marker-end", "url(#ArrowEnd)")
                .set("d", skel_data);
        document = document.add(skel_path);
    }
	// only save when float coords done ok
	svg::save(format!("{}/{}", BASEPATH, filename), &document).unwrap();
}
