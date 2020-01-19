use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};
use nphysics3d::object::DefaultBodySet;
use nphysics3d::object::DefaultColliderSet;
use nphysics3d::object::RigidBodyDesc;
use na::Vector3;
use ncollide3d::shape::{ShapeHandle, Cuboid, Ball};
use nphysics3d::object::ColliderDesc;
use nphysics3d::object::BodyPartHandle;
use nphysics3d::object::DefaultBodyHandle;
use std::collections::HashMap;
// use nphysics3d::object::Collider;
use nphysics3d::object::Body;
use nphysics3d::math::Isometry;
use nphysics3d::object::RigidBody;
use nalgebra::geometry::Isometry3;


use crate::util;


#[allow(dead_code)]
pub struct Entity {
    color: (f32, f32, f32),
    handle: DefaultBodyHandle,
    collider: nphysics3d::object::DefaultColliderHandle,
}


#[allow(dead_code)]
pub struct World {
    next_id: usize,
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    entities: HashMap<usize, Entity>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

impl World {
    #[allow(dead_code)]
    pub fn new() -> World {
        World {
            next_id: 1,
            mechanical_world: DefaultMechanicalWorld::new(Vector3::new(0.0, -10.0, 0.0)),
            geometrical_world: DefaultGeometricalWorld::new(),
            entities: HashMap::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_body_mut(&mut self, id: usize) -> &mut dyn Body<f32> {
        self.bodies.get_mut(self.entities[&id].handle).unwrap()
    }

    #[allow(dead_code)]
    pub fn get_rigid_mut(&mut self, id: usize) -> &mut RigidBody<f32> {
        self.bodies.rigid_body_mut(self.entities[&id].handle).unwrap()
    }
    #[allow(dead_code)]
    pub fn get_rigid(&self, id: usize) -> &RigidBody<f32> {
        self.bodies.rigid_body(self.entities[&id].handle).unwrap()
    }

    #[allow(dead_code)]
    pub fn add_cube(&mut self, size: (f32, f32, f32)) -> usize {
        use nphysics3d::material::{MaterialHandle, BasicMaterial};
        use nalgebra::base::Matrix3;
        // let mut node = window.add_cube(size.0, size.1, size.2);
        // node.set_color(rf(0.8, 1.0), rf(0.8, 1.0), rf(0.5, 1.0));

        let body = RigidBodyDesc::<f32>::new()
                            // .angular_damping(1.0)
                            .angular_inertia(Matrix3::new_rotation(0.0))
                            .set_mass(1.0)
                            .build();

        let handle = self.bodies.insert(body);
        let shape = ShapeHandle::new(Cuboid::new(Vector3::new(size.0, size.1, size.2)));

        let collider = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.1, 4.0)))
            // .set_rotation(Vector3::new(0.0, 0.1, 0.4))
            .build(BodyPartHandle(handle, 0));
        let coll_handle = self.colliders.insert(collider);

        self.add_entity(Entity {
            color: util::rand_color(),
            handle,
            collider: coll_handle,
        })
    }

    #[allow(dead_code)]
    pub fn add_ball(&mut self, size: f32) -> usize {
        use nphysics3d::material::{MaterialHandle, BasicMaterial};
        use nalgebra::base::Matrix3;
        // let mut node = window.add_sphere(size);
        // node.set_color(rf(0.8, 1.0), rf(0.8, 1.0), rf(0.5, 1.0));

        let body = RigidBodyDesc::<f32>::new()
                            // .linear_damping(10.0)
                            // .angular_damping(5.0)
                            .angular_damping(5.0)
                            .angular_inertia(Matrix3::new_rotation(0.0))
                            .set_mass(2.0)
                            .build();

        let handle = self.bodies.insert(body);
        let shape = ShapeHandle::new(Ball::new(size));

        let collider = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.2, 10.0)))
            // .set_rotation(Vector3::new(0.0, 1.0, 2.0))
            .build(BodyPartHandle(handle, 0));
        let coll_handle = self.colliders.insert(collider);

        self.add_entity(Entity {
            color: util::rand_color(),
            handle,
            collider: coll_handle,
        })
    }

    #[allow(dead_code)]
    pub fn add_entity(&mut self, e: Entity) -> usize {
        let id = self.next_id;
        self.next_id+= 1;
        self.entities.insert(id, e);
        id
    }

    #[allow(dead_code)]
    pub fn update_physics(&mut self) {
        self.geometrical_world.sync_colliders(&self.bodies, &mut self.colliders);
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators
        );
    }

    #[allow(dead_code)]
    pub fn set_iso(&mut self, id: usize, pos: Isometry3<f32>) {
        self.get_rigid_mut(id).set_position(pos);
    }
    #[allow(dead_code)]
    pub fn get_iso(&self, id: usize) -> Isometry3<f32> {
        let c = self.colliders.get(self.entities[&id].collider).unwrap();
        let r = self.get_rigid(id);
        Isometry3::from_parts(
            r.position().translation,
            c.position().rotation,
        )
    }
    #[allow(dead_code)]
    pub fn set_pos(&mut self, id: usize, pos: &(f32, f32, f32)) {
        self.get_rigid_mut(id).set_position(Isometry::translation(pos.0, pos.1, pos.2));
    }

    #[allow(dead_code)]
    pub fn ent_mut(&mut self, id: usize) -> &mut Entity {
        self.entities.get_mut(&id).unwrap()
    }
}
