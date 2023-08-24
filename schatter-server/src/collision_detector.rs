use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use parry3d::na::{
    distance, Isometry3, Point3, Quaternion, Rotation, Rotation3, Translation, Translation3,
    UnitQuaternion, Vector3,
};
use parry3d::query::PointQuery;
use parry3d::shape::Cone;

pub struct CollisionDetector {
    circle_center: Point3<f32>,
    circle_radius: f32,
    viewpoint: Point3<f32>,
}

impl CollisionDetector {
    pub fn new() -> CollisionDetector {
        CollisionDetector {
            circle_center: Point3::new(-20.0, 1920.0, 0.0),
            circle_radius: 400.0,
            viewpoint: Point3::new(-2000.0, 1920.0, 10000.0),
        }
    }

    pub fn detect_collison(
        &self,
        viewpoint: Point3<f32>,
        point: &Point3<f32>,
        circle_radius: f32,
    ) -> bool {
        let d = distance(&self.circle_center, &viewpoint);
        // println!("{}", d);
        let ci_r = circle_radius;
        let co_r = (ci_r.powf(2.0) - (ci_r.powf(4.0) / d.powf(2.0))).sqrt();
        // println!("co_r {}", co_r);
        let co_h = d - (ci_r.powf(2.0) / d);
        // println!("co_h {}", co_h);
        let cone = Cone::new(co_h, co_r * 2.0);

        let test = Point3::new(0.0, -1.0, 0.0);
        // let inside = cone.contains_point(&Isometry3::identity(), &test);
        // println!("{} ", inside);
        let translation = Translation3::new(0.0, 0.0, co_h);
        // let inside = cone.contains_point(&Isometry3::translation(0.0, 0.0, co_h), &test);
        // println!("Translated {} ", inside);
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -FRAC_PI_2);
        let cone_along_z = Isometry3::from_parts(translation, rotation);
        // let test = Point3::new(1.0, 0.0, 0.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("x+ false {}", inside);
        // let test = Point3::new(-1.0, 0.0, 0.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("x- false {}", inside);
        // let test = Point3::new(0.0, 1.0, 0.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("y+ false {}", inside);
        // let test = Point3::new(0.0, -1.0, 0.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("y- false {}", inside);
        // let test = Point3::new(0.0, 0.0, 1.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("z+ true {}", inside);
        // let test = Point3::new(0.0, 0.0, -1.0);
        // let inside = cone.contains_point(&cone_along_z, &test);
        // println!("z- false {}", inside);

        let view = Isometry3::face_towards(&viewpoint, &self.circle_center, &Vector3::y_axis());

        let isometry = view * cone_along_z;

        // println!("Translated and rotated {}", inside);
        cone.contains_point(&isometry, &point)
    }
}
