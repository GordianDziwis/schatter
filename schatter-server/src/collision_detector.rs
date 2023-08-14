use parry3d::na::Point3;

struct CollisionDetector {
    viewpoint: Point3<f32>,
    circle_center: Point3<f32>,
    circle_diameter: f32,
}

impl CollisionDetector {
    pub fn new() -> CollisionDetector {
        CollisionDetector {
            viewpoint: Point3::new(0.0, 300.0, 10000.0),
            circle_center: Point3::new(-20.0, 1920.0, 0.0),

            circle_diameter: 300.0,
        }
    }

    pub fn detect_collison(viewpoint: Point3<f32>, point: Point3<f32>) {
    }
}
