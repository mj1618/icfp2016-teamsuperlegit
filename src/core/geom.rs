use super::*;

use super::super::matrix::Matrix33;

use std::collections::BTreeMap;

#[derive(Debug,Clone,PartialOrd)]
pub struct Point<N: Num> {
	pub x: N,
	pub y: N,
}

#[derive(Debug,Clone,PartialEq)]
pub struct Line<N: Num> {
	pub p1: Point<N>,
	pub p2: Point<N>
}

#[derive(Debug,Clone,PartialEq)]
pub struct Polygon<N: Num> {
	pub points: Vec<Point<N>>,
	pub transform: Matrix33<N>
}

#[derive(Debug,Clone)]
pub struct Shape<N: Num> {
	pub polys: Vec<Polygon<N>>,
}

#[derive(Debug,Clone)]
pub struct Skeleton<N: Num> {
	pub lines: Vec<Line<N>>,
}

// infinite line intersection. Returns the intersection point or None if the
// lines do not intercept.
//
// An epsilon is used to mark lines that are very close to parallel as parallel.
pub fn intersect_inf<N:Num>(a: &Line<N>, b: &Line<N>) -> Option<Point<N>> {
	let x1 = a.p1.x.clone();
	let y1 = a.p1.y.clone();
	let x2 = a.p2.x.clone();
	let y2 = a.p2.y.clone();
	let x3 = b.p1.x.clone();
	let y3 = b.p1.y.clone();
	let x4 = b.p2.x.clone();
	let y4 = b.p2.y.clone();

  // If the lines are very close to parallel return None
  let d = (x1.clone() - x2.clone())*(y3.clone() - y4.clone()) - (y1.clone() - y2.clone())*(x3.clone() - x4.clone());
  if eq_eps(&d, &N::zero()) {
    return None;
  }

  let x_out = ((x1.clone()*y2.clone() - y1.clone()*x2.clone())*(x3.clone() - x4.clone()) - (x1.clone() - x2.clone())*(x3.clone()*y4.clone() - y3.clone()*x4.clone())) / d.clone();
  let y_out = ((x1.clone()*y2.clone() - y1.clone()*x2.clone())*(y3.clone() - y4.clone()) - (y1.clone() - y2.clone())*(x3.clone()*y4.clone() - y3.clone()*x4.clone())) / d.clone();

  Some(Point{x: x_out, y: y_out})
}

fn cross_scalar<N: Num>(a: &Point<N>, b: &Point<N>) -> N {
	a.x.clone() * b.y.clone() - a.y.clone() * b.x.clone()
}

// http://stackoverflow.com/a/1968345
// discrete line intersection
// 
// Returns the intersection point, or None if the lines do not intercept.
pub fn intersect_discrete<N: Num>(a: &Line<N>, b: &Line<N>) -> Option<Point<N>> {
	let s1 = &a.p2 - &a.p1;
	let s2 = &b.p2 - &b.p1;
	let c1 = &a.p1 - &b.p1;

	let x = divide( cross_scalar(&s1, &c1), cross_scalar(&s1, &s2) );
	let y = divide( cross_scalar(&s2, &c1), cross_scalar(&s1, &s2) );
    
    if x==None || y==None{
        return None;
    }
    
    let s = x.unwrap();
    let t = y.unwrap();

	if (s >= N::zero()) && (s < N::one()) && (t >= N::zero()) && (t <= N::one()) {
		return Some(&a.p1 + s1.scale(t));
	}

	None
}



// Use intersect_poly_inf or _discrete below instead of this function
fn intersect_poly<N: Num>(line: Line<N>, other: &Polygon<N>, discrete: bool) -> Vec<(Point<N>, Point<N>)> {
	let mut candidates = Vec::new();
	for boundary in other.edges().iter() {
		// Check normal intersections
		let point: Option<Point<N>>;
		if discrete {
			point = intersect_discrete(&line, &boundary);
		} else {
			point = intersect_inf(&line, &boundary);
		}

		if let Some(p) = point {
			// The proposed intersection must be coincident on the boundary (ie. the
			// discrete segment only). intersect_inf will give us inf intersect for
			// both lines - we only want the input line to be infinite.
			if boundary.coincident(&p) {
				//println!("intersect_poly - adding candidate {} from intersection of {} and {}", p, line, boundary);
				candidates.push(p);
			} else {
				//println!("intersect_poly - candidate {} is not conincident on boundary {}, skipping", p, boundary);
			}
		}
	}

	candidates.sort_by_key(|p| (&line).dist_along(p).to_rat());
	candidates.dedup();
	let mut segments = Vec::new();
	if candidates.len() % 2 != 0 {
		println!("intersect_poly: {} {:?}", line, candidates);
		return segments;
	}
	for i in 0..candidates.len()/2 {
		segments.push((candidates[2*i].clone(), candidates[2*i + 1].clone()));
	}
	segments
}

// Return the points where a line intersects the given poly (discrete lines). 
//
// If the line does not intersect the returned vector is empty
pub fn intersect_poly_discrete<N:Num>(line: Line<N>, other: &Polygon<N>) -> Vec<(Point<N>, Point<N>)> {
	intersect_poly(line, other, true)
}

// Return the points where a line intersects the given poly, if it is
// extended to infinity in both directions
//
// If the line does not intersect the returned vector is empty
pub fn intersect_poly_inf<N:Num>(line: Line<N>, other: &Polygon<N>) -> Vec<(Point<N>, Point<N>)> {
	intersect_poly(line, other, false)
}

pub fn gradient<N:Num>(l: &Line<N>) -> Option<N> {
	divide(  l.p2.y.clone() - l.p1.y.clone(),  l.p2.x.clone() - l.p1.x.clone() )
}

pub fn reflect_matrix<N:Num>(vertex1: &Point<N>, vertex2: &Point<N>) -> Matrix33<N> {
    let l = Line::new(vertex1.clone(),vertex2.clone());
    
    let m = match gradient(&l) {
        Some(g) =>{
            let c = vertex1.y.clone() - g.clone() * vertex1.x.clone();
            let d = vertex2 - vertex1;

            Matrix33::translate(N::zero(),-c.clone())
                .then_rotate( - d.clone().y / v_distance(&d), d.clone().x / v_distance(&d) )
                .then_scale(N::one(),N::from_f64(-1.0))
                .then_rotate( d.clone().y / v_distance(&d), d.clone().x / v_distance(&d) )
                .then_translate(N::zero(),c.clone())
        }
        None => {
            Matrix33::rotate(N::one(),N::zero())
                .then_scale(N::one(),N::from_f64(-1.0))
                .then_rotate(N::from_f64(-1.0),N::zero())
        }
    };

    //println!("{}", m);
    return m;
}

//flips both points of a line on an axis
pub fn flip_line<N:Num>(line: &Line<N>, vertex1: &Point<N>, vertex2: &Point<N>) -> Line<N> {
	let affine = reflect_matrix(&vertex1,&vertex2);
	Line{ p1: affine.transform(line.p1.clone()), p2: affine.transform(line.p2.clone()) }
}

// If there is an intersection, assume line.p1 is the point that does not get flipped
pub fn fold_line<N:Num>(line: &Line<N>, vertex1: &Point<N>, vertex2: &Point<N>) -> Vec<Line<N>> {
	let intersect = intersect_discrete(&line,&Line{p1:vertex1.clone(),p2:vertex2.clone()});

	match intersect {
		Some(p) => {
			let l1 = Line{p1: p.clone(), p2: line.p1.clone() };
			let l2 = Line{p1: p.clone(), p2: reflect_matrix(&vertex1,&vertex2).transform(line.p2.clone()) };
			vec!(l1,l2)
		}
        None => vec!(flip_line(&line,&vertex1,&vertex2))
	}
}

pub fn flip_polygon<N: Num>(poly: &Polygon<N>, vertex1: &Point<N>, vertex2: &Point<N>) -> Polygon<N> {
	let mut poly_f = Vec::new();
	let affine = reflect_matrix(&vertex1,&vertex2);

	for pt in poly.clone().points {
		poly_f.push(affine.transform(pt.clone()));
	}
	poly_f.reverse();
	let mut ret = Polygon::new(poly_f);
	ret.transform = poly.transform.clone() * affine;
	ret
}

pub fn fold_polygon<N: Num>(poly: &Polygon<N>, vertex1: &Point<N>, vertex2: &Point<N>, anchor: &Point<N>) -> Vec<Polygon<N>> {
	let mut polys = split_polygon(&poly,&vertex1,&vertex2);
	for poly in polys.iter_mut() {
		if !poly.contains(anchor) {
			*poly = flip_polygon(&poly, &vertex1, &vertex2);
		}
	}
	polys
}

pub fn split_polygon<N: Num>(poly: &Polygon<N>, v1: &Point<N>, v2: &Point<N>) -> Vec<Polygon<N>> {
	let mut polys = Vec::new();
	let mut resume = BTreeMap::new();
	let fold = Line::new(v1.clone(), v2.clone());

	let intersections = intersect_poly_inf(fold, poly);
	//println!("intersections: {:?}", intersections);

	polys.push(Vec::new());
	let mut cur = 0;

	for edge in poly.edges() {
		//println!("starting edge {}; on poly {}/{}", edge, cur+1, polys.len());
		polys[cur].push(edge.p1.clone());

		let mut co = None;
		for i in 0..intersections.len() {
			if edge.coincident(&intersections[i].0) && edge.p1 != intersections[i].0 {
				co = Some(intersections[i].clone());
				break;
			} else if edge.coincident(&intersections[i].1) && edge.p1 != intersections[i].1 {
				co = Some((intersections[i].1.clone(), intersections[i].0.clone()));
				break;
			}
		}

		if let Some((a,b)) = co {
			//println!("intersected point {}; poly {} now waiting for point {}", a, cur+1, b);
			polys[cur].push(a.clone());
			resume.insert(b.clone(), cur);

			// switch to next polygon
			cur = match resume.get(&a) {
				Some(p) => *p,
				None => {
					polys.push(Vec::new());
					polys.len() - 1
				}
			};
			if edge.p2 != a {
				polys[cur].push(a.clone());
			}
		}
	}

	polys.into_iter().map(|points| Polygon::with_transform(points, poly.transform.clone())).collect()
}

pub fn p_distance<N: Num>(p1: &Point<N>, p2: &Point<N>) -> N {
	v_distance(&(p1 - p2))
}

pub fn v_distance<N: Num>(p: &Point<N>) -> N {
	if p.x == N::zero() { p.y.abs() }
	else if p.y == N::zero() { p.x.abs() }
	else { N::from_f64((p.x.clone() * p.x.clone() + p.y.clone() * p.y.clone()).to_f64().sqrt()) }
}


pub fn normalize_line<N:Num>(start: &Point<N>, dir: &Point<N>) -> Point<N> {
	let ratio = N::one() / v_distance(dir);
	let scaled = dir.scale(ratio);
	start + &scaled
}

impl Point<f64> {
	pub fn to_num<N: Num>(&self) -> Point<N> {
		Point{x: N::from_f64(self.x), y: N::from_f64(self.y)}
	}
}

impl<N: Num> Point<N> {
	pub fn to_f64(&self) -> Point<f64> {
		Point{x: self.x.to_f64(), y: self.y.to_f64()}
	}

	pub fn scale(&self, alpha: N) -> Point<N> {
		Point{x: self.x.clone() * alpha.clone(), y: self.y.clone() * alpha}
	}

	pub fn dot(&self, other: Point<N>) -> N {
		self.x.clone()*other.x.clone() + self.y.clone()*other.y.clone()
	}
}

impl<N: Num> Polygon<N> {
	pub fn new(points: Vec<Point<N>>) -> Polygon<N> {
		// transform is setup to do nothing by default
		// should represent the transformation to go back to unit square
		Polygon{points: points, transform: Matrix33::identity()}
	}

	pub fn with_transform(points: Vec<Point<N>>, transform: Matrix33<N>) -> Polygon<N> {
		Polygon{points: points, transform: transform}
	}

	fn double_signed_area(&self) -> f64 {
		let mut sum = N::zero();
		for edge in self.edges() {
			sum = sum + (edge.p2.x.clone() - edge.p1.x.clone()) * (edge.p2.y.clone() + edge.p1.y.clone());
		}
		return sum.to_f64();
	}

	pub fn area(&self) -> f64 {
		return (self.double_signed_area() / 2.0_f64).abs();
	}

	pub fn printcongruency(&self) {
		let mut p = self.edges().last().unwrap().clone();
		for edge in self.edges() {
			let (u, v) = (&p.p2 - &p.p1, &edge.p2 - &edge.p1);
			let angle = (u.dot(v.clone()) / (v_distance(&u) * v_distance(&v))).to_f64().acos().to_degrees();
			print!(" {}° ", angle);
			print!("<{} -> {}>({})", edge.p1, edge.p2, edge.len());
			p = edge;
		}
		println!("");
	}

	pub fn source_poly(&self) -> Polygon<N> {
		let affine = self.transform.inverse();
		let mut points = Vec::new();
		for p in self.points.iter() {
			points.push(affine.transform(p.clone()));
		}
		let mut poly = Polygon::new(points);
		poly.transform = affine;
		poly
	}

	/* returns true where the poly points are in clockwise order,
	** based on area contained within. thx to:
	** http://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order */
	pub fn is_hole(&self) -> bool {
		return self.double_signed_area() >= 0.0;
	}

	pub fn square(&self) -> bool {
		if self.corners().len() == 4 {
			if self.edges().len() == 4{
				return true
			}
		}
		return false
	}

	pub fn corners(&self) -> Vec<(Line<N>, Line<N>)> {
		let edges = self.edges();
		let mut corners: Vec<(Line<N>, Line<N>)> = Vec::new();
		let mut previous = edges.len() - 1;
		for (i, edge) in edges.iter().enumerate() {
			let edge1 = edges[previous].clone();
			let cornerangle = (angle(&edge1.p1, &edge1.p2) - angle(&edge.p1, &edge.p2)).abs();
			if cornerangle % 90.0_f64.to_radians() < 0.00001 {
				corners.push((edge1.clone(), edge.clone()));
			}
			previous = i;
		}
		return corners;
	}

	// Return this polygon as a vector of lines
	pub fn edges(&self) -> Vec<Line<N>> {
		let mut edges: Vec<Line<N>> = Vec::new();
		let mut previous = self.points.len() - 1;
		for (i, point) in self.points.iter().enumerate() {
			let edge = Line{p1: self.points[previous].clone(), p2: point.clone()};
			edges.push(edge);
			previous = i;
		}
		return edges;
	}

	// Test whether point contained within this polygon
	pub fn contains(&self, test: &Point<N>) -> bool {
		self.inside(test) || self.coincident(test)
	}

	pub fn inside(&self, test: &Point<N>) -> bool {
		// https://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
		let end = self.points.len();
		let mut contains = false;
		for offset in 0..end {
			//println!("contains - offset={}/{}", offset, end);
			let ref p1 = self.points[offset];
			let ref p2 = self.points[(offset+1)%end];
			let intersect = ((p1.y.clone() > test.y.clone()) != (p2.y.clone() > test.y.clone())) &&
				(test.x.clone() < (p2.x.clone() - p1.x.clone())*(test.y.clone() - p1.y.clone()) / (p2.y.clone() - p1.y.clone()) + p1.x.clone());
			if intersect {
				//println!("intersect");
				contains = !contains;
			}
		}

		contains
	}

	// Test whether point coincident on this polygon
	pub fn coincident(&self, test: &Point<N>) -> bool {
		let end = self.points.len();
		for offset in 0..end {
			let l_test = Line::new(self.points[offset].clone(), self.points[(offset+1)%end].clone());
			if l_test.coincident(&test) {
				return true
			}
		}
		false
	}

	// Return the set of edges of this polygon that slice the provided polygon.
	//
	// An edge qualifies if it
	//  - crosses at least one boundary of the unit square.
	//  - lies wholly within the unit square
	pub fn slicey_edges(self, other: Polygon<N>) -> Vec<Line<N>> {
		let mut candidates = Vec::new();

		for edge in self.edges() {
			// println!("slicey_edges - considering line {}", edge);
			// println!("  contained {} {}", other.contains(&edge.p1.to_f64()), other.contains(&edge.p2.to_f64()));
			let mut intersection = intersect_poly_discrete(edge.clone(), &other);
			if intersection.len() == 0 {
				// Line lies wholly within or wholly without the unit square, or straddles the boundary
				if other.contains(&edge.p1) || other.contains(&edge.p2) {
					intersection = intersect_poly_inf(edge.clone(), &other);
				}
			}
			// Poss. do something with intersection here
			
			if intersection.len() != 0 {
				candidates.append(&mut intersection.into_iter().map(|(p1, p2)| Line::new(p1, p2)).collect());
			}
		}

		candidates
	}
}

impl<N: Num> Shape<N> {
	pub fn new(polys: Vec<Polygon<N>>) -> Shape<N> {
		Shape{polys: polys}
	}

	pub fn area(self) -> f64 {
		let mut a = 0.0;
		for p in self.polys {
			let sgn = if p.is_hole() { -1.0 } else { 1.0 };
			a += sgn * p.area();
		}
		a
	}
}

impl<N: Num> Line<N> {
	pub fn new(p1: Point<N>, p2: Point<N>) -> Line<N> {
		return Line{p1: p1, p2: p2};
	}

	// Returns the length of this line
	pub fn len(&self) -> N {
		return p_distance(&self.p1, &self.p2);
	}

	// True if point lies on this line
	pub fn coincident(&self, point: &Point<N>) -> bool {
		return eq_eps(&(p_distance(&self.p1, point) + p_distance(point, &self.p2)), &self.len());
	}

	// Returns a point along this line. 0 <= alpha <= 1, else you're extrapolating bro
	pub fn interpolate(&self, alpha: N) -> Point<N> {
		&self.p1 + &(&self.p2 - &self.p1).scale(alpha)
	}

	// Splits this line into two at the specified position along it.
	pub fn split(&self, alpha: N) -> (Line<N>, Line<N>) {
		let mid = self.interpolate(alpha);
		let l1 = Line::new(self.p1.clone(), mid.clone());
		let l2 = Line::new(mid, self.p2.clone());
		(l1, l2)
	}

	// Returns how far along this line the specified point is. Assumes point is coincident.
	pub fn dist_along(&self, p: &Point<N>) -> N {
		p_distance(&self.p1, p) / self.len()
	}
}

impl<N: Num> Skeleton<N> {
	pub fn new(lines: Vec<Line<N>>) -> Skeleton<N> {
		return Skeleton{lines: lines};
	}

	pub fn push(self, line: Line<N>) -> Skeleton<N> {
		let mut lines: Vec<Line<N>> = self.lines.clone();
		lines.push(line);
		return Skeleton{lines: lines};
	}

	pub fn lines(self) -> Vec<Line<N>> {
		return self.lines.clone();
	}

	// Returns the number of lines composing this skeleton
	pub fn len(self) -> usize {
		return self.lines.len();
	}
}

pub fn angle<'a, N: Num>(p0: &'a Point<N>, p1: &'a Point<N>) -> f64 {
	let d = p1 - p0;
	return d.x.to_f64().atan2(d.y.to_f64());
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::tests::*;

	#[test]
	fn test_point_eq(){
    assert_eq!(p(0,0), p(0,0));
    assert_eq!(p(0.0,0.0), p(0.0,0.0));
    assert_eq!(p(0.0,0.0), p(0.0000000001,0.0000000001));
    assert_eq!(p(1.0,1.0), p(1.0000000001,1.0000000001));

    assert!(!(p(1,1) == p(0,0)));
    assert!(!(p(1.0,1.0) == p(0.0000000001,0.0000000001)));
  }

	#[test]
	fn gradient_test(){
        let p1 = p(1,1);
        let p2 = p(0,0);
        let g = gradient(&Line{p1:p1,p2:p2});
        println!("gradient_test: {:?}",g);
        assert_eq!(g,Some(1));
	}

	#[test]
	fn test_flip_point_matrix(){
		let mut p2 = reflect_matrix(&p(0.0,0.0), &p(1.0,0.0)).transform(p(1.0,1.0));
		println!("flip_point_test: {:?}",p2);
		assert_eq!(p(1.0, -1.0), p2);

		p2 = reflect_matrix(&p(0.0,0.0), &p(3.0,3.0)).transform(p(1.0,0.0));
		println!("flip_point_test: {:?}",p2);
		assert_eq!(p(0.0, 1.0), p2);

		p2 = reflect_matrix(&p(0.0,0.0), &p(0.866025403784439,0.5)).transform(p(1.0,0.0)); // unit vector along x, 30 deg line. Result should be unit vector 60 degrees to the x axis
		println!("flip_point_test: {:?}",p2);

		p2 = reflect_matrix(&p(0.0,0.0), &p(0.0,3.0)).transform(p(-1.0,1.0));
		println!("flip_point_test: {:?}",p2);
		assert_eq!(p(1.0, 1.0), p2);
	}

	#[test]
	fn flip_line_test(){
        let l1 = Line::new(p(0.0,2.0),p(0.0,3.0));
        let v1 = p(1.0,1.0);
        let v2 = p(2.0,2.0);
        let l2 = flip_line(&l1,&v1,&v2);
        println!("flip_line_test: {:?}",l2);
        
        assert_eq!(Line::new(p(2.0,0.0),p(3.0,0.0)), l2);
	}
    #[test]
    fn fold_line_test(){
        
        let l1 = Line::new(p(0.0,2.0),p(2.0,0.0));
        let v1 = p(0.0,0.0);
        let v2 = p(2.0,2.0);
        let l2 = fold_line(&l1,&v1,&v2);
        println!("fold_line_test: {:?}",l2);
        
        assert_eq!(vec!(Line::new(p(1.0,1.0),p(0.0,2.0)),Line::new(p(1.0,1.0),p(0.0,2.0))), l2);
    }

	#[test]
	fn fold_polygon_test(){
		let poly = Polygon::new(vec!( p(0.0,0.0),p(2.0,0.0),p(2.0,2.0),p(0.0,2.0) ));
		let v1 = p(0.0,1.0);
		let v2 = p(2.0,1.0);
		let ret = fold_polygon(&poly,&v1,&v2,&p(0.0, 2.0));

		println!("fold_polygon_test: {:?}",ret);

		let ans = vec!( p(0.0,1.0),p(0.0,2.0),p(2.0,2.0),p(2.0,1.0));
		for pt in ans {
			assert!(ret[0].points.contains(&pt));
			assert!(ret[1].points.contains(&pt));
		}
		assert!(ret.len() == 2);
	}
    
	#[test]
	fn test_intersect_discrete() {
		let l1 = Line::<f64>{p1: p(0.0, 0.0), p2: p(1.0, 1.0)};
		let l2 = Line::<f64>{p1: p(0.0, 1.0), p2: p(1.0, 0.0)};
		assert_eq!(intersect_discrete(&l1,&l2).unwrap(), p(0.5, 0.5));

		let l1 = Line::<f64>{p1: p(0.0, 0.0), p2: p(0.25, 0.25)};
		let l2 = Line::<f64>{p1: p(0.0, 1.0), p2: p(1.0, 0.0)};
		assert_eq!(intersect_discrete(&l1,&l2), None);
	}

	#[test]
	fn test_intersect_infinite() {
		let l1 = Line::new(p(0.1, 0.3), p(0.25, 0.75));
		let l2 = Line::new(p(1.0, 0.0), p(1.0, 1.0));
		assert_eq!(intersect_inf(&l1,&l2).unwrap(), p(1.0, 3.0));

		let l1 = Line::new(p(2.0, 0.3), p(2.0, 0.75));
		let l2 = Line::new(p(1.0, 0.0), p(1.0, 1.0));
		assert_eq!(intersect_inf(&l1,&l2), None);

		let l1 = Line::new(p(0.1, 0.3), p(0.25, 0.75));
		let l2 = Line::new(p(0.0, 1.0), p(1.0, 1.0));
		let l3 = Line::new(p(1.0, 1.0), p(1.0, 0.0));
		let l4 = Line::new(p(1.0, 0.0), p(0.0, 0.0));
		let l5 = Line::new(p(0.0, 0.0), p(0.0, 1.0));
		assert_eq!(p(0.33333333333333337, 1.0), intersect_inf(&l1, &l2).unwrap());
		assert_eq!(p(1.0, 3.0), intersect_inf(&l1, &l3).unwrap());
		assert_eq!(p(0.0, 0.0), intersect_inf(&l1, &l4).unwrap());
		assert_eq!(p(0.0, 0.0), intersect_inf(&l1, &l5).unwrap());
	}

	#[test]
	fn test_angle() {
		assert_eq!(45.0.to_radians(), angle(&p(0, 0), &p(1, 1)));
		assert_eq!(-135.0.to_radians(), angle(&p(1, 1), &p(0, 0)));
	}

	#[test]
	fn test_clockwise() {
		assert!(!Polygon::new(vec!(p(0, 0), p(1, 0), p(1, 1), p(0, 1))).is_hole());
		assert!(Polygon::new(vec!(p(0, 0), p(0, 1), p(1, 1), p(1, 0))).is_hole());
		assert!(Polygon::new(vec!(p(1, 1), p(1, 2), p(2, 2), p(2, 1))).is_hole());
	}

	#[test]
	fn test_area() {
		assert_eq!(1.0, Polygon::new(vec!(p(0, 0), p(1, 0), p(1, 1), p(0, 1))).area());
		assert_eq!(1.0, Polygon::new(vec!(p(0, 0), p(0, 1), p(1, 1), p(1, 0))).area());
		let p22 = Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2)));
		assert_eq!(4.0, p22.area());
		let p44 = Polygon::new(vec!(p(0, 0), p(4, 0), p(4, 4), p(0, 4)));
		let hole12 = Polygon::new(vec!(p(1, 1), p(1, 2), p(2, 2), p(2, 1)));
		assert!(hole12.is_hole());
		assert_eq!(15.0, Shape::new(vec!(p44, hole12)).area());
	}

	#[test]
	fn test_line_coincident() {
		assert!(Line::new(p(0,0), p(0,10)).coincident(&p(0,5)));
		assert!(Line::new(p(0,0), p(0,10)).coincident(&p(0,0)));
		assert!(Line::new(p(0,0), p(0,10)).coincident(&p(0,10)));
		assert!(Line::new(p(0.0,0.0), p(0.0,10.0)).coincident(&p(0.0,0.0)));
		assert!(Line::new(p(0.0,0.0), p(0.0,10.0)).coincident(&p(0.0,10.0)));
		assert!(Line::new(p(-4.0,0.0), p(0.0,-4.0)).coincident(&p(-2.875,-1.125)));
		assert!(!Line::new(p(0.0,0.0), p(0.0,10.0)).coincident(&p(1.0,5.0)));
		assert!(!Line::new(p(0.0,0.0), p(0.0,10.0)).coincident(&p(0.0,11.0)));
	}

	#[test]
	fn test_poly_contains() {
		assert!(Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).contains(&p(0,0)));
		assert!(Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).contains(&p(1,0)));
		assert!(Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).contains(&p(1,1)));
		assert!(!Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).contains(&p(3,3)));

		assert!(Polygon::new(vec!(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0))).contains(&p(0.2,0.2)));
		assert!(Polygon::new(vec!(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0))).contains(&p(0.2,0.7)));
		assert!(Polygon::new(vec!(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0))).contains(&p(0.7,0.7)));
		assert!(!Polygon::new(vec!(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0))).contains(&p(1.3,0.7)));
	}

	#[test]
	fn test_poly_coincident() {
		assert!(Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).coincident(&p(0,0)));
		assert!(Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).coincident(&p(1,0)));
		assert!(!Polygon::new(vec!(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 2.0), p(0.0, 2.0))).coincident(&p(1.0,1.0)));
		assert!(!Polygon::new(vec!(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 2.0), p(0.0, 2.0))).coincident(&p(3.0,3.0)));
	}

	#[test]
	fn test_contains_3() {
		assert!(!Polygon::new(vec!(p(0, 0), p(2, 0), p(2, 2), p(0, 2))).contains(&p(3,3)));
	}

	#[test]
	fn test_split() {
		let (start, mid, end) = (p(1.0, 1.5), p(1.125, 2.0), p(1.25, 2.5));
		let line1 = Line::new(start.clone(), end.clone());
		assert_eq!((Line::new(start.clone(), mid.clone()), Line::new(mid.clone(), end.clone())), line1.split(0.5))
	}

	#[test]
	fn test_interpolate() {
		let line1 = Line::new(p(0.0, 0.0), p(1.0, 3.0));
		assert_eq!(p(0.5, 1.5), line1.interpolate(0.5));
		assert_eq!(p(0.125, 0.375), line1.interpolate(0.125));
	}

	#[test]
	// also exercises intersect_inf and intersect_discrete
	fn test_intersect_poly() {
		let unit_sq_p = Polygon::new(vec![Point{x: 0.0, y: 0.0}, Point{x: 0.0, y: 1.0}, Point{x:1.0, y: 1.0}, Point{x: 1.0, y: 0.0}]);

		let line1 = Line::new(p(2.0, 0.0), p(1.0, 3.0));
		assert_eq!(0, intersect_poly_discrete(line1, &unit_sq_p).len());

		let line2 = Line::new(p(0.0, 0.0), p(1.0, 3.0));
		assert_eq!(vec![(Point { x: 0.0, y: 0.0 }, Point { x: 0.3333333333333333, y: 1.0 })], intersect_poly_discrete(line2, &unit_sq_p));

		let line3 = Line::new(p(0.1, 0.3), p(0.25, 0.75));
		assert_eq!(vec![(Point { x: 0.0, y: 0.0 }, Point { x: 0.33333333333333337, y: 1.0 })], intersect_poly_inf(line3, &unit_sq_p));

		let line4 = Line::new(p(0.0, 0.0), p(1.0, 3.0));
		assert_eq!(vec![(Point { x: 0.0, y: 0.0 }, Point { x: 0.3333333333333333, y: 1.0 })], intersect_poly_inf(line4, &unit_sq_p));

		let line5 = Line::new(p(2.0, 0.0), p(1.0, 3.0));
		assert_eq!(0, intersect_poly_inf(line5, &unit_sq_p).len());
	}

	#[test]
	fn test_slicey_edges() {
		let unit_sq_p = Polygon::new(vec![Point{x: 0.0, y: 0.0}, Point{x: 0.0, y: 1.0}, Point{x:1.0, y: 1.0}, Point{x: 1.0, y: 0.0}]);
		let base = Polygon::new(vec!(p(-4.0, 0.0), p(0.0, -4.0), p(4.0, 0.0), p(0.0, 4.0)));

		println!("## Rotated square base, silhouette as above");
		let mut a = Polygon::new(vec!(p(0.0, 0.0), p(0.5, 0.0), p(2.0, 0.5), p(0.5, 0.5))).slicey_edges(base.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(4, a.len());

		println!("## Rotated square base, some inside some out");
		a = Polygon::new(vec!(p(0.0, 0.0), p(10.0, -10.0), p(11.0, 0.5), p(5.0, 5.0))).slicey_edges(base.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(2, a.len());

    	// Unit base
		println!("## Polygon with vertices on unit sq corners/parallel lines");
		a = Polygon::new(vec!(p(0.0, 0.0), p(0.5, 0.0), p(2.0, 0.5), p(0.5, 0.5))).slicey_edges(unit_sq_p.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(4, a.len());

		println!("## 'normal' polygon, some inside some out");
		a = Polygon::new(vec!(p(-1.3, -1.2), p(0.5, -0.5), p(2.0, 0.5), p(0.5, 0.5))).slicey_edges(unit_sq_p.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(2, a.len());

		println!("## Polygon surrounds the unit sq");
		a = Polygon::new(vec!(p(-1.0, -1.0), p(1.5, -0.5), p(1.5, 1.5), p(-1.0, 1.5))).slicey_edges(unit_sq_p.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(0, a.len());

		println!("## Polygon contained within the unit");
		a = Polygon::new(vec!(p(0.2, 0.2), p(0.7, 0.2), p(0.7, 0.7), p(0.2, 0.7))).slicey_edges(unit_sq_p.clone());
		println!("Number of intersecting edges: {}", a.len());
		for edge in a.clone() {
			println!("{}", edge);
		}
		assert_eq!(4, a.len());
	}

	#[test]
	fn test_iterateedges() {
		let poly = Polygon::new(vec!(p(0, 0), p(1, 0), p(2, 2), p(0, 1)));
		for (i, _) in poly.edges().iter().enumerate() {
			assert!(i < poly.edges().len());
		}
	}

	#[test]
	fn test_normalize_line() {
		let (p1, p2) = (p(1.0, 1.5), p(0.5, 0.0));
		assert_eq!(p(2.0, 1.5), normalize_line(&p1, &p2));
	}
}
