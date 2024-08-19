use glam::Vec3;
use super::ICoords;

// 1/(sqrt(N)/2), N=3 -> 2/sqrt(3)
// sqrt() is not a const function, so use a high-precision value instead.
// TODO: Replace fixed const values with const fn if sqrt() ever becomes a const function.
// 2/sqrt(3) = 1.1547005383792515290182975610039149112952035025402537520372046529
const SCALE_FACTOR: f64 = 1.154_700_538_379_251_5;

#[inline]
fn linear(a: f32, b: f32, alpha: f32) -> f32 {
    b * alpha + a * (1. - alpha)
}

#[inline]
fn smoothstep(x: f32) -> f32 {
    3. * x * x - 2. * x * x * x
}

#[inline]
fn dot_product(a: &ICoords, b: &Vec3) -> f32 {
    a.x as f32 * b.x + a.y as f32 * b.y + a.z as f32 * b.z
}

// returns the pseudorandom gradient vector at grid point p
#[inline]
fn gradient(p: &ICoords) -> ICoords {
    // pseudorandom hash value of `p` that is hopefully
    // quick to compute
    let mut hash = p.x - p.y - p.z;
    hash = hash ^ (hash >> 32);
    hash = hash ^ (hash >> 16);
    hash = hash ^ (hash >> 8);
    hash = hash ^ (hash >> 4);
    hash = hash & 0b1111;

    match hash {
         0 | 12 => ICoords { x: 0, y: 1, z: 1 },
         1      => ICoords { x: 0, y: 1, z: -1 },
         2 | 14 => ICoords { x: 0, y: -1, z: 1 },
         3      => ICoords { x: 0, y: -1, z: -1 },
         4      => ICoords { x: 1, y: 1, z: 0 },
         5      => ICoords { x: 1, y: -1, z: 0 },
         6 | 13 => ICoords { x: -1, y: 1, z: 0 },
         7      => ICoords { x: -1, y: -1, z: 0 },
         8      => ICoords { x: 1, y: 0, z: 1 },
         9      => ICoords { x: 1, y: 0, z: -1 },
        10      => ICoords { x: -1, y: 0, z: 1 },
        11 | 15 => ICoords { x: -1, y: 0, z: -1 },
        _ => unreachable!()
    }
}

#[inline]
fn gradient_cross(p: Vec3, corner: ICoords) -> f32 {
    let grad = gradient(&(ICoords::from_vec3(p) + corner));
    // vector from corner to p
    let corner_to_p: Vec3 = p - p.floor() - corner.vec3();
    dot_product(&grad, &corner_to_p)
}

// perlin-like noise function
pub fn perlin(p: Vec3) -> f32 {
    let v000 = gradient_cross(p, ICoords{ x: 0, y: 0, z: 0 });
    let v001 = gradient_cross(p, ICoords{ x: 0, y: 0, z: 1 });
    let v010 = gradient_cross(p, ICoords{ x: 0, y: 1, z: 0 });
    let v011 = gradient_cross(p, ICoords{ x: 0, y: 1, z: 1 });
    let v100 = gradient_cross(p, ICoords{ x: 1, y: 0, z: 0 });
    let v101 = gradient_cross(p, ICoords{ x: 1, y: 0, z: 1 });
    let v110 = gradient_cross(p, ICoords{ x: 1, y: 1, z: 0 });
    let v111 = gradient_cross(p, ICoords{ x: 1, y: 1, z: 1 });

    let delta = p - p.floor();
    let alpha = Vec3::new(smoothstep(delta.x), smoothstep(delta.y), smoothstep(delta.z));

    linear(
        linear(
            linear(v000, v001, alpha.z), 
            linear(v010, v011, alpha.z), 
            alpha.y
        ), 
        linear(
            linear(v100, v101, alpha.z),
            linear(v110, v111, alpha.z),
            alpha.y
        ), 
    alpha.x) / SCALE_FACTOR as f32
}

pub fn visualize(granularity: f32) {
    for i in 0..100 {
        let x = granularity * i as f32;
        let v = perlin(Vec3::new(3. + x, 0., 0.));
        print!("{:<5.2}", x);
        if v < 0. {
            let line = "-".repeat((-v * 20.) as usize);
            println!("{:>20}", line);
        } else {
            let line = "+".repeat((v * 20.) as usize);
            println!("{:<20}{}", "", line);
        }
    }
}