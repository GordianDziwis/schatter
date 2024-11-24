use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use parry3d::na::{
    distance, Isometry3, Point3, Quaternion, Rotation, Rotation3, Translation, Translation3,
    UnitQuaternion, Vector3,
};
use parry3d::query::PointQuery;
use parry3d::shape::Cone;

pub struct CollisionDetector {
    circle_center: Point3<f32>,
}

impl CollisionDetector {
    pub fn new() -> CollisionDetector {
        CollisionDetector {
            circle_center: Point3::new(-20.0, 1920.0, 0.0),
        }
    }

    pub fn detect_collison(
        &self,
        viewpoint: Point3<f32>,
        point: &Point3<f32>,
        pupil_radius: f32,
    ) -> bool {
        let d = distance(&self.circle_center, &viewpoint);
        let e_r = 3000.0;
        let pu_r = pupil_radius;
        let co_r = d * pu_r / (d - e_r);
        let cone = Cone::new(d, co_r);
        let translation = Translation3::new(0.0, 0.0, d);
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -FRAC_PI_2);
        let cone_along_z = Isometry3::from_parts(translation, rotation);
        let view = Isometry3::face_towards(&viewpoint, &self.circle_center, &Vector3::y_axis());
        let isometry = view * cone_along_z;
        cone.contains_point(&isometry, &point)
    }
}
