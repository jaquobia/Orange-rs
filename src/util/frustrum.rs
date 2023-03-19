use ultraviolet::{Mat4, Vec3, Vec4};

pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    fn isOnOrForwardPlane(plane: &FrustrumPlane, center: Vec3, extents: Vec3) -> bool {
        let r = extents.x * plane.normal.x.abs() + extents.y * plane.normal.y.abs() + extents.z * plane.normal.z.abs();
        plane.getSignedDistanceToPlane(center).abs() >= -r
    }

    // pub fn isOnFrustum(&self, frustrum: Frustrum) -> bool {
    //
    // }
}

pub struct Frustrum {
    planes: [FrustrumPlane; 6],
}

impl Frustrum {
    pub fn new(origin: Vec3, front: Vec3, right: Vec3, up: Vec3, aspect: f32, fovY: f32, znear: f32, zfar: f32) -> Self {
        let half_vside = zfar * (fovY * 0.5).tan();
        let half_hside = half_vside * aspect;
        let front_mult_far = zfar * front;

        let planes: [FrustrumPlane; 6] = [
            FrustrumPlane::new(origin + znear * front, front),
            FrustrumPlane::new(origin + front_mult_far, -front),
            FrustrumPlane::new(origin, (front_mult_far - (right * half_hside)).cross(up)),
            FrustrumPlane::new(origin, up.cross(front_mult_far + (right * half_hside))),
            FrustrumPlane::new(origin, right.cross(front_mult_far - (up * half_vside))),
            FrustrumPlane::new(origin, (front_mult_far + (up * half_vside)).cross(right)),
        ];

        Self {
            planes,
        }
    }

    pub fn aabb_intersects(&self, min: Vec3, max: Vec3, front: Vec3, right: Vec3, up: Vec3) -> bool {
        let center = (max + min) * 0.5;
        let extents = max - center;

        // let x_axis = Vec3::new(1.0, 0.0, 0.0);
        // let y_axis = Vec3::new(0.0, 1.0, 0.0);
        // let z_axis = Vec3::new(0.0, 0.0, 1.0);
        //
        // let right = right * extents.x;
        // let up = up * extents.y;
        // let front = front * extents.z;

        // let extents = Vec3::new(
        //     x_axis.dot(right).abs() + x_axis.dot(up).abs() + x_axis.dot(front).abs(),
        //     y_axis.dot(right).abs() + y_axis.dot(up).abs() + y_axis.dot(front).abs(),
        //     z_axis.dot(right).abs() + z_axis.dot(up).abs() + z_axis.dot(front).abs(),
        // );
        // let center: Vec3 = (*model_mat * Vec4::new(center.x, center.y, center.z, 1.0)).truncated();

        AABB::isOnOrForwardPlane(&self.planes[3], center, extents) &&
            AABB::isOnOrForwardPlane(&self.planes[2], center, extents) &&
            AABB::isOnOrForwardPlane(&self.planes[0], center, extents) &&
            AABB::isOnOrForwardPlane(&self.planes[1], center, extents) &&
            AABB::isOnOrForwardPlane(&self.planes[5], center, extents) &&
            AABB::isOnOrForwardPlane(&self.planes[4], center, extents)
    }
}

struct FrustrumPlane {
    normal: Vec3,
    distance: f32,
}

impl FrustrumPlane {
    fn new(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalized();
        let distance = normal.dot(point);
        Self { normal , distance }
    }

    fn getSignedDistanceToPlane(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}