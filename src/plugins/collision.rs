use bevy::prelude::*;

use crate::{
    collider::{get_aabb_projection_radius, get_axes, get_obb_projection_radius, Collider},
    walls::{Direction, Wall, WallType},
    Velocity,
};

use super::{shooting::Bullet, tank::Tank};

#[derive(Component)]
pub struct Static;

#[derive(Component)]
pub struct Dynamic;

type OnlyDynamic = (With<Dynamic>, Without<Static>);

fn update_bounds_system(mut query: Query<(&Transform, &mut Collider), OnlyDynamic>) {
    for (transform, mut collider) in query.iter_mut() {
        match *collider {
            Collider::Aabb(ref mut aabb) => {
                let xyz = transform.translation.xyz();
                aabb.center = xyz.into();
            }
            Collider::Obb(ref mut obb) => {
                let angle = transform.rotation.to_euler(EulerRot::XYZ).2.to_degrees();
                let xy = transform.translation.xy();
                obb.center = xy;
                obb.rotation = angle;
            }
        }
    }
}

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_bounds_system,
                bullet_wall_collision_system,
                tank_wall_collision_system,
            )
                .chain(),
        );
    }
}

fn reflect_angle(angle: f32, wall_type: WallType) -> f32 {
    match wall_type {
        WallType::Horizontal => -angle,
        WallType::Vertical => 180.0 - angle,
    }
}

fn bullet_wall_collision_system(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &Collider, &mut Bullet, &mut Transform)>,
    wall_query: Query<(&Collider, &Wall)>,
) {
    for (bullet_entity, bullet_collider, mut bullet, mut bullet_transform) in
        bullet_query.iter_mut()
    {
        let mut collided = false;

        for (wall_collider, wall) in wall_query.iter() {
            let wall_aabb = match wall_collider {
                Collider::Obb(_) => {
                    return;
                }
                Collider::Aabb(a) => a,
            };

            let is_same_wall = match bullet.last_hit_wall {
                Some(wall) => wall == wall_aabb.center.into(),
                None => false,
            };

            if is_same_wall {
                continue;
            }
            let collision = bullet_collider.collides_with(wall_collider);

            if collision {
                let new_angle = reflect_angle(bullet.angle, wall.wall_type.clone());
                bullet.angle = new_angle;
                bullet.last_hit_wall = Some(wall_aabb.center.into());

                let normal = match wall.wall_type {
                    WallType::Horizontal => {
                        Vec2::new(0.0, if bullet.velocity().y > 0.0 { -1.0 } else { 1.0 })
                    }
                    WallType::Vertical => {
                        Vec2::new(if bullet.velocity().x > 0.0 { -1.0 } else { 1.0 }, 0.0)
                    }
                };
                let adjustment = normal * 0.1;
                bullet_transform.translation.x += adjustment.x;
                bullet_transform.translation.y += adjustment.y;

                collided = true;
                break;
            }
        }

        if collided {
            bullet.bounce_count += 1;
            if bullet.bounce_count > 5 {
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}
fn tank_wall_collision_system(
    mut tank_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Tank>>,
    wall_query: Query<(&Collider, &Direction)>,
) {
    for (mut velocity, mut transform, tank_collider) in tank_query.iter_mut() {
        for (wall_collider, direction) in wall_query.iter() {
            if tank_collider.collides_with(wall_collider) {
                // Get direction vector based on wall direction
                let push_direction = match direction {
                    Direction::Up => Vec2::new(0.0, -1.0),
                    Direction::Down => Vec2::new(0.0, 1.0),
                    Direction::Left => Vec2::new(1.0, 0.0),
                    Direction::Right => Vec2::new(-1.0, 0.0),
                };

                // Push tank slightly away from wall to prevent sticking
                transform.translation += (push_direction * 5.0).extend(0.0);
            }
        }
    }
}