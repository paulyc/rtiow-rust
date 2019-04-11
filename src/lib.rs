pub mod camera;
mod material;
mod object;
pub mod ray;
pub mod vec3;

use rand::prelude::*;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::material::Material;
use crate::object::{hit_slice, Object};
use crate::ray::Ray;
use crate::vec3::{Axis::*, Channel::*, *};

/// Computes the pixel color along `ray` for the scene of objects `world`.
///
/// This is the actual ray-tracing routine.
pub fn color(world: &[Object], mut ray: Ray, rng: &mut impl Rng) -> Vec3 {
    let mut strength = Vec3::from(1.);
    let mut bounces = 0;

    while let Some(hit) = hit_slice(world, &ray) {
        if bounces < 50 {
            if let Some((new_ray, attenuation)) = hit.material.scatter(&ray, &hit, rng) {
                ray = new_ray;
                strength = strength * attenuation;
                bounces += 1;
                continue;
            }
        }
        return Vec3::default();
    }

    let unit_direction = ray.direction.into_unit();
    let t = 0.5 * (unit_direction[Y] + 1.0);
    let col = (1. - t) * Vec3::from(1.) + t * Vec3(0.5, 0.7, 1.0);
    strength * col
}

pub fn random_scene(rng: &mut impl Rng) -> Vec<Object> {
    let mut world = vec![Object::Sphere {
        center: Vec3(0., -1000., 0.),
        radius: 1000.,
        material: Material::Lambertian {
            albedo: Vec3::from(0.5),
        },
    }];

    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if (center - Vec3(4., 0.2, 0.)).length() > 0.9 {
                let choose_mat = rng.gen::<f32>();

                let obj = if choose_mat < 0.8 {
                    Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Lambertian {
                            albedo: rng.gen::<Vec3>() * rng.gen::<Vec3>(),
                        },
                    }
                } else if choose_mat < 0.95 {
                    Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Metal {
                            albedo: 0.5 * (1. + rng.gen::<Vec3>()),
                            fuzz: 0.5 * rng.gen::<f32>(),
                        },
                    }
                } else {
                    Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Dielectric { ref_idx: 1.5 },
                    }
                };
                world.push(obj);
            }
        }
    }

    world.push(Object::Sphere {
        center: Vec3(0., 1., 0.),
        radius: 1.0,
        material: Material::Dielectric { ref_idx: 1.5 },
    });

    world.push(Object::Sphere {
        center: Vec3(-4., 1., 0.),
        radius: 1.0,
        material: Material::Lambertian {
            albedo: Vec3(0.4, 0.2, 0.1),
        },
    });

    world.push(Object::Sphere {
        center: Vec3(4., 1., 0.),
        radius: 1.0,
        material: Material::Metal {
            albedo: Vec3(0.7, 0.6, 0.5),
            fuzz: 0.,
        },
    });

    world
}

pub struct Image(Vec<Vec<Vec3>>);

impl Image {
    pub fn par_compute(nx: usize, ny: usize, f: impl Fn(usize, usize) -> Vec3 + Sync) -> Image {
        Image(
            (0..ny)
                .into_par_iter()
                .rev()
                .map(|y| (0..nx).map(|x| f(x, y)).collect())
                .collect(),
        )
    }

    pub fn compute(nx: usize, ny: usize, mut f: impl FnMut(usize, usize) -> Vec3) -> Image {
        Image(
            (0..ny)
                .rev()
                .map(|y| (0..nx).map(|x| f(x, y)).collect())
                .collect(),
        )
    }
}

pub fn print_ppm(image: Image) {
    println!("P3\n{} {}\n255", image.0[0].len(), image.0.len());
    for scanline in image.0 {
        for col in scanline {
            let col = Vec3(col.0.sqrt(), col.1.sqrt(), col.2.sqrt());

            let ir = (255.99 * col[R]) as i32;
            let ig = (255.99 * col[G]) as i32;
            let ib = (255.99 * col[B]) as i32;

            println!("{} {} {}", ir, ig, ib);
        }
    }
}

pub fn par_cast(nx: usize, ny: usize, ns: usize, camera: &Camera, world: &[Object]) -> Image {
    Image::par_compute(nx, ny, |x, y| {
        let col: Vec3 = (0..ns)
            .map(|_| {
                let mut rng = rand::thread_rng();
                let u = (x as f32 + rng.gen::<f32>()) / nx as f32;
                let v = (y as f32 + rng.gen::<f32>()) / ny as f32;
                let r = camera.get_ray(u, v, &mut rng);
                color(&world, r, &mut rng)
            })
            .sum();
        col / ns as f32
    })
}

pub fn cast(
    nx: usize,
    ny: usize,
    ns: usize,
    camera: &Camera,
    world: &[Object],
    rng: &mut impl Rng,
) -> Image {
    Image::compute(nx, ny, |x, y| {
        let col: Vec3 = (0..ns)
            .map(|_| {
                let u = (x as f32 + rng.gen::<f32>()) / nx as f32;
                let v = (y as f32 + rng.gen::<f32>()) / ny as f32;
                let r = camera.get_ray(u, v, rng);
                color(&world, r, rng)
            })
            .sum();
        col / ns as f32
    })
}
