//! 3D Spatial Surface Representation & Raycast Hit Testing (Milestone 0002, ADR-0010, ADR-0012).
//!
//! Replaces traditional 2D overlapping application windows with fluid planar
//! surfaces existing along a curved 3D perspective depth with rack focus.
//! Deterministic ray-plane unprojection ensures 100% reliable pointer capture.

use glam::{Mat4, Vec2, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(pub u64);

/// Physical and spatial transform for a surface in Arc's continuous 3D canvas.
#[derive(Debug, Clone)]
pub struct SpatialSurface {
    pub id: SurfaceId,
    pub title: String,
    /// World position [x, y, z] in the 3D canvas coordinate system.
    pub position: Vec3,
    /// Rotation angles [pitch, yaw, roll] in radians.
    pub rotation: Vec3,
    /// Physical planar dimensions [width, height] in world units.
    pub size: Vec2,
    /// Depth of field focus weight: 1.0 = razor-sharp foreground, 0.0 = completely blurred background.
    pub focus: f32,
    /// Visual opacity [0.0 - 1.0].
    pub opacity: f32,
}

impl SpatialSurface {
    pub fn new(id: SurfaceId, title: String, width: f32, height: f32) -> Self {
        Self {
            id,
            title,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            size: Vec2::new(width, height),
            focus: 1.0,
            opacity: 1.0,
        }
    }

    /// Calculates the 4x4 model transform matrix for this surface.
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_rotation_y(self.rotation.y)
            * Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_rotation_z(self.rotation.z)
            * Mat4::from_scale(Vec3::new(self.size.x, self.size.y, 1.0))
    }

    /// Ray-plane intersection test for 3D input arbitration (ADR-0012 §3).
    /// Returns the distance along the ray and local UV coordinates [0.0..1.0] if hit.
    pub fn intersect_ray(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<(f32, Vec2)> {
        // Normal of the planar surface quad (local +Z in world space)
        let rot_mat = Mat4::from_rotation_y(self.rotation.y)
            * Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_rotation_z(self.rotation.z);
        let normal = rot_mat.transform_vector3(Vec3::Z).normalize();

        let denom = normal.dot(ray_dir);
        if denom.abs() < 1e-6 {
            return None; // Parallel to plane
        }

        let p0_to_origin = self.position - ray_origin;
        let t = p0_to_origin.dot(normal) / denom;
        if t < 0.0 {
            return None; // Surface is behind the ray
        }

        let hit_world = ray_origin + ray_dir * t;
        let hit_local = self.model_matrix().inverse().transform_point3(hit_world);

        // Quad spans [-0.5..0.5] in local space
        if hit_local.x >= -0.5 && hit_local.x <= 0.5 && hit_local.y >= -0.5 && hit_local.y <= 0.5 {
            let uv = Vec2::new(hit_local.x + 0.5, 0.5 - hit_local.y);
            Some((t, uv))
        } else {
            None
        }
    }
}

/// Perspective virtual camera viewing the 3D spatial continuum.
#[derive(Debug, Clone)]
pub struct SpatialCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_rad: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl SpatialCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1000.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y_rad: 45.0_f32.to_radians(),
            aspect,
            z_near: 1.0,
            z_far: 10000.0,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y_rad, self.aspect, self.z_near, self.z_far)
    }

    /// Unprojects a 2D screen coordinate (pixels, y-down) into a 3D ray in world space (ADR-0012 §3).
    pub fn screen_point_to_ray(&self, screen_pos: Vec2, screen_size: Vec2) -> (Vec3, Vec3) {
        // Convert screen pixel position to Normalized Device Coordinates (NDC) [-1.0..1.0]
        let ndc_x = (2.0 * screen_pos.x) / screen_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_size.y;

        let inv_view_proj = (self.projection_matrix() * self.view_matrix()).inverse();

        let near_ndc = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_view_proj * near_ndc;
        let far_world = inv_view_proj * far_ndc;

        let ray_origin = near_world.truncate() / near_world.w;
        let ray_endpoint = far_world.truncate() / far_world.w;
        let ray_dir = (ray_endpoint - ray_origin).normalize();

        (ray_origin, ray_dir)
    }
}

/// Raycast-driven hit test arbiter (ADR-0012 §3).
pub fn arbitrate_pointer_hit<'a>(
    surfaces: &'a [SpatialSurface],
    ray_origin: Vec3,
    ray_dir: Vec3,
) -> Option<(&'a SpatialSurface, f32, Vec2)> {
    let mut closest: Option<(&'a SpatialSurface, f32, Vec2)> = None;

    for surf in surfaces {
        if surf.opacity < 0.01 {
            continue; // Transparent interaction blockers rejected per REQ-SCENE-004
        }
        if let Some((t, uv)) = surf.intersect_ray(ray_origin, ray_dir) {
            match closest {
                Some((_, prev_t, _)) if t < prev_t => {
                    closest = Some((surf, t, uv));
                }
                None => {
                    closest = Some((surf, t, uv));
                }
                _ => {}
            }
        }
    }

    closest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_plane_intersection_direct_hit() {
        let mut surface = SpatialSurface::new(SurfaceId(1), "Test Card".into(), 400.0, 300.0);
        surface.position = Vec3::new(0.0, 0.0, 0.0);

        let camera = SpatialCamera::new(16.0 / 9.0);
        // Cast ray from center of screen (960, 540) on a 1920x1080 display
        let (ray_origin, ray_dir) =
            camera.screen_point_to_ray(Vec2::new(960.0, 540.0), Vec2::new(1920.0, 1080.0));

        let hit = surface.intersect_ray(ray_origin, ray_dir);
        assert!(hit.is_some(), "Ray through center should hit surface at origin");
        let (t, uv) = hit.unwrap();
        assert!(t > 0.0);
        assert!((uv.x - 0.5).abs() < 1e-3);
        assert!((uv.y - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_hit_arbitration_closest_surface_wins() {
        let mut front_surface = SpatialSurface::new(SurfaceId(1), "Front".into(), 400.0, 300.0);
        front_surface.position = Vec3::new(0.0, 0.0, 200.0); // closer to camera at Z=1000

        let mut back_surface = SpatialSurface::new(SurfaceId(2), "Back".into(), 400.0, 300.0);
        back_surface.position = Vec3::new(0.0, 0.0, 0.0); // farther

        let surfaces = vec![back_surface, front_surface];

        let camera = SpatialCamera::new(16.0 / 9.0);
        let (ray_origin, ray_dir) =
            camera.screen_point_to_ray(Vec2::new(960.0, 540.0), Vec2::new(1920.0, 1080.0));

        let arb = arbitrate_pointer_hit(&surfaces, ray_origin, ray_dir);
        assert!(arb.is_some());
        let (winner, _, _) = arb.unwrap();
        assert_eq!(winner.id, SurfaceId(1), "Foremost surface with lower distance must win arbitration");
    }

    #[test]
    fn test_transparent_blocker_ignored() {
        let mut ghost_surface = SpatialSurface::new(SurfaceId(1), "Invisible Blocker".into(), 400.0, 300.0);
        ghost_surface.position = Vec3::new(0.0, 0.0, 200.0);
        ghost_surface.opacity = 0.0; // Invisible / clickjack attempt

        let mut real_surface = SpatialSurface::new(SurfaceId(2), "Legitimate Target".into(), 400.0, 300.0);
        real_surface.position = Vec3::new(0.0, 0.0, 0.0);
        real_surface.opacity = 1.0;

        let surfaces = vec![ghost_surface, real_surface];

        let camera = SpatialCamera::new(16.0 / 9.0);
        let (ray_origin, ray_dir) =
            camera.screen_point_to_ray(Vec2::new(960.0, 540.0), Vec2::new(1920.0, 1080.0));

        let arb = arbitrate_pointer_hit(&surfaces, ray_origin, ray_dir);
        assert!(arb.is_some());
        let (winner, _, _) = arb.unwrap();
        assert_eq!(winner.id, SurfaceId(2), "Transparent clickjack blocker must be rejected per REQ-SCENE-004");
    }
}
