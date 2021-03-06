use std::io::{Error,Write};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use num::rational::BigRational;
use num::{BigInt, One, Zero};
use num::ToPrimitive;

use core::*;

pub trait Folds<N: Num> {
	// Given a source point, returns its final destination after applying all the folds
	fn transform(&self, src: &Point<N>) -> Point<N>;
}

// XXX unfinished
fn facets<N: Num>(skel: Skeleton<N>) -> (Vec<Point<N>>, Vec<Vec<usize>>) {
	let mut points = Vec::new();
	let facets = Vec::new();
	/* 1. find edges which share a vertex
	** 2. sort edges according to angle
	** 3. construct poly using shortest line segments along adjacent angles */
	for i in 0..skel.lines.len() {
		for j in i+1..skel.lines.len() {
			if let Some(p) = intersect_discrete(&skel.lines[i], &skel.lines[j]) {
				points.push(p);
			}
		}
	}
	for line in skel.lines {
		points.push(line.p1.clone());
		points.push(line.p2.clone());
	}
	//points.sort_by(|a, b| if (a < b) { Ordering::Less } else if (a > b) { Ordering::Greater } else { Ordering::Equal }});
	points.sort();
	points.dedup();

	(points, facets)
}

#[allow(dead_code)]
pub fn from_skeleton<N: Num, F: Folds<N>, W: Write>(writer: W, skel: Skeleton<N>, folds: F) -> Result<(), Error> {
	let (points, facets) = facets(skel);
	let mut dst = Vec::new();
	for p in points.iter() {
		dst.push(folds.transform(p));
	}
	write(writer, points, facets, dst)
}

fn snap(p: Point<BigRational>) -> Point<BigRational> {
	let mut p = p.clone();
	let snapdist = 0.000000001;
	for (float, snap) in vec![(0.0, Zero::zero()),(1.0, One::one())] {
		let snap: BigRational = snap;
		p.x = if (p.x.to_f64() - float).abs() < snapdist { snap.clone() } else { p.x };
		p.y = if (p.y.to_f64() - float).abs() < snapdist { snap.clone() } else { p.y };
	}
	return p;
}

fn qntz(p: Point<BigRational>, base: BigInt) -> Point<BigRational> {
	use num::bigint::BigInt;
	let p = p.clone();
	let xnum = (p.x.to_f64() * base.to_f64().unwrap()).round() as i64;
	let ynum = (p.y.to_f64() * base.to_f64().unwrap()).round() as i64;
	let p2 = Point{
		x: BigRational::new(BigInt::from(xnum), BigInt::from(base.clone())),
		y: BigRational::new(BigInt::from(ynum), BigInt::from(base.clone()))
	};
	if (p.x.to_f64() - p2.x.to_f64()).abs() < 0.000000001 && (p.y.to_f64() - p2.y.to_f64()).abs() < 0.000000001 {
		return p2;
	} else {
		return p;
	}
}

pub fn from_polys<W: Write>(writer: W, polys: Vec<Polygon<BigRational>>, base: BigInt) -> Result<Vec<Polygon<BigRational>>, Error> {
	let mut seen = BTreeMap::new();
	let mut src = Vec::new();
	let mut dst: Vec<Point<BigRational>> = Vec::new();
	let mut facets = Vec::new();
	let mut unfolded: Vec<Polygon<BigRational>> = Vec::new();
	for poly in polys {
		//poly.printcongruency();
		//poly.source_poly().printcongruency();
		let mut facet = Vec::new();
		let mut orig = Vec::new();
		for point in poly.points {
			let p = find_close_rational_point(qntz(snap(point.clone()), base.clone()));
			let i = {
				let e = seen.entry(p.clone());
				match e {
					Entry::Occupied(e) => {
						*e.get()
					},
					Entry::Vacant(e) => {
						src.push(find_close_rational_point(qntz(snap(poly.transform.inverse().transform(p.clone())), base.clone())));
						dst.push(p.clone());
						let i = dst.len() - 1;
						println!("   POINT {} {} -> {}", i, src[i], dst[i]);
						*e.insert(i)
					}
				}
			};
			facet.push(i);
			orig.push(src[i].clone());
		}
		facets.push(facet);
		unfolded.push(Polygon::new(orig));
	}
	write(writer, src, facets, dst).unwrap();
	return Ok(unfolded);
}

// currently private but may be a better entry point in the future?
// `points` is the list of vertices that make up the facet corners
// `facets` is a list of integer sequences, where each integer is an index into `points`
fn write<N: Num, W: Write>(mut writer: W, src: Vec<Point<N>>, facets: Vec<Vec<usize>>, dst: Vec<Point<N>>) -> Result<(), Error> {
	assert_eq!(src.len(), dst.len());
	try!(write!(writer, "{}\n", src.len()));
	for p in src {
		try!(write!(writer, "{}\n", find_close_rational_point(p)));
	}
	try!(write!(writer, "{}\n", facets.len()));
	for facet in facets {
		try!(write!(writer, "{} ", facet.len()));
		for index in facet {
			try!(write!(writer, "{} ", index));
		}
		try!(write!(writer, "\n"));
	}
	for p in dst {
		try!(write!(writer, "{}\n", find_close_rational_point(p)));
	}
	Ok(())
}
