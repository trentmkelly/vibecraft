use super::*;

pub fn random_next_i32_bound(random: &mut RandomSourceKind, bound: i32) -> i32 {
    match random {
        RandomSourceKind::Legacy(random) => random.next_i32_bound(bound),
        RandomSourceKind::Xoroshiro(random) => random.next_i32_bound(bound),
    }
}

pub fn random_next_i64(random: &mut RandomSourceKind) -> i64 {
    match random {
        RandomSourceKind::Legacy(random) => random.next_i64(),
        RandomSourceKind::Xoroshiro(random) => random.next_i64(),
    }
}

pub fn random_next_f64(random: &mut RandomSourceKind) -> f64 {
    match random {
        RandomSourceKind::Legacy(random) => random.next_f64(),
        RandomSourceKind::Xoroshiro(random) => random.next_f64(),
    }
}

pub fn feature_random_next_i32_bound(random: &mut RandomSourceKind, bound: i32) -> i32 {
    worldgen_random_next_i32_bound(random, bound)
}

pub fn feature_random_next_i64(random: &mut RandomSourceKind) -> i64 {
    worldgen_random_next_i64(random)
}

pub fn feature_random_next_f32(random: &mut RandomSourceKind) -> f32 {
    worldgen_random_next_f32(random)
}

pub fn feature_random_next_f64(random: &mut RandomSourceKind) -> f64 {
    worldgen_random_next_f64(random)
}

pub fn improved_noise_snapshot(random: &mut RandomSourceKind) -> ImprovedNoiseSnapshot {
    let xo = random_next_f64(random) * 256.0;
    let yo = random_next_f64(random) * 256.0;
    let zo = random_next_f64(random) * 256.0;
    let mut permutation = [0_u8; 256];
    for (index, value) in permutation.iter_mut().enumerate() {
        *value = index as u8;
    }
    for index in 0..256 {
        let offset = random_next_i32_bound(random, 256 - index as i32) as usize;
        permutation.swap(index, index + offset);
    }

    ImprovedNoiseSnapshot {
        xo,
        yo,
        zo,
        permutation,
    }
}

fn smoothstep(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

fn smoothstep_derivative(x: f64) -> f64 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

pub fn lerp(alpha: f64, p0: f64, p1: f64) -> f64 {
    p0 + alpha * (p1 - p0)
}

fn lerp2(alpha1: f64, alpha2: f64, x00: f64, x10: f64, x01: f64, x11: f64) -> f64 {
    lerp(alpha2, lerp(alpha1, x00, x10), lerp(alpha1, x01, x11))
}

#[allow(clippy::too_many_arguments)]
pub fn lerp3(
    alpha1: f64,
    alpha2: f64,
    alpha3: f64,
    x000: f64,
    x100: f64,
    x010: f64,
    x110: f64,
    x001: f64,
    x101: f64,
    x011: f64,
    x111: f64,
) -> f64 {
    lerp(
        alpha3,
        lerp2(alpha1, alpha2, x000, x100, x010, x110),
        lerp2(alpha1, alpha2, x001, x101, x011, x111),
    )
}

pub fn gradient_dot(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    let gradient = SIMPLEX_GRADIENT[(hash & 15) as usize];
    f64::from(gradient[0]) * x + f64::from(gradient[1]) * y + f64::from(gradient[2]) * z
}

fn simplex_gradient(hash: u8) -> [i32; 3] {
    SIMPLEX_GRADIENT[(hash & 15) as usize]
}

fn simplex_dot(gradient: [i32; 3], x: f64, y: f64, z: f64) -> f64 {
    f64::from(gradient[0]) * x + f64::from(gradient[1]) * y + f64::from(gradient[2]) * z
}

fn improved_noise_permutation(snapshot: &ImprovedNoiseSnapshot, x: i32) -> u8 {
    snapshot.permutation[(x & 0xff) as usize]
}

fn improved_noise_corner_hashes(
    snapshot: &ImprovedNoiseSnapshot,
    xf: i32,
    yf: i32,
    zf: i32,
) -> [u8; 8] {
    let x0 = i32::from(improved_noise_permutation(snapshot, xf));
    let x1 = i32::from(improved_noise_permutation(snapshot, xf + 1));
    let xy00 = i32::from(improved_noise_permutation(snapshot, x0 + yf));
    let xy01 = i32::from(improved_noise_permutation(snapshot, x0 + yf + 1));
    let xy10 = i32::from(improved_noise_permutation(snapshot, x1 + yf));
    let xy11 = i32::from(improved_noise_permutation(snapshot, x1 + yf + 1));
    [
        improved_noise_permutation(snapshot, xy00 + zf),
        improved_noise_permutation(snapshot, xy10 + zf),
        improved_noise_permutation(snapshot, xy01 + zf),
        improved_noise_permutation(snapshot, xy11 + zf),
        improved_noise_permutation(snapshot, xy00 + zf + 1),
        improved_noise_permutation(snapshot, xy10 + zf + 1),
        improved_noise_permutation(snapshot, xy01 + zf + 1),
        improved_noise_permutation(snapshot, xy11 + zf + 1),
    ]
}

fn lerp3_corners(alphas: [f64; 3], values: [f64; 8]) -> f64 {
    lerp3(
        alphas[0], alphas[1], alphas[2], values[0], values[1], values[2], values[3], values[4],
        values[5], values[6], values[7],
    )
}

pub fn improved_noise_sample(
    snapshot: &ImprovedNoiseSnapshot,
    input_x: f64,
    input_y: f64,
    input_z: f64,
    y_scale: f64,
    y_fudge: f64,
) -> f64 {
    let x = input_x + snapshot.xo;
    let y = input_y + snapshot.yo;
    let z = input_z + snapshot.zo;
    let xf = x.floor() as i32;
    let yf = y.floor() as i32;
    let zf = z.floor() as i32;
    let xr = x - f64::from(xf);
    let yr = y - f64::from(yf);
    let zr = z - f64::from(zf);
    let yr_fudge = if y_scale != 0.0 {
        let fudge_limit = if y_fudge >= 0.0 && y_fudge < yr {
            y_fudge
        } else {
            yr
        };
        (fudge_limit / y_scale + 1.0E-7).floor() * y_scale
    } else {
        0.0
    };

    let x0 = i32::from(improved_noise_permutation(snapshot, xf));
    let x1 = i32::from(improved_noise_permutation(snapshot, xf + 1));
    let xy00 = i32::from(improved_noise_permutation(snapshot, x0 + yf));
    let xy01 = i32::from(improved_noise_permutation(snapshot, x0 + yf + 1));
    let xy10 = i32::from(improved_noise_permutation(snapshot, x1 + yf));
    let xy11 = i32::from(improved_noise_permutation(snapshot, x1 + yf + 1));
    let d000 = gradient_dot(
        improved_noise_permutation(snapshot, xy00 + zf),
        xr,
        yr - yr_fudge,
        zr,
    );
    let d100 = gradient_dot(
        improved_noise_permutation(snapshot, xy10 + zf),
        xr - 1.0,
        yr - yr_fudge,
        zr,
    );
    let d010 = gradient_dot(
        improved_noise_permutation(snapshot, xy01 + zf),
        xr,
        yr - yr_fudge - 1.0,
        zr,
    );
    let d110 = gradient_dot(
        improved_noise_permutation(snapshot, xy11 + zf),
        xr - 1.0,
        yr - yr_fudge - 1.0,
        zr,
    );
    let d001 = gradient_dot(
        improved_noise_permutation(snapshot, xy00 + zf + 1),
        xr,
        yr - yr_fudge,
        zr - 1.0,
    );
    let d101 = gradient_dot(
        improved_noise_permutation(snapshot, xy10 + zf + 1),
        xr - 1.0,
        yr - yr_fudge,
        zr - 1.0,
    );
    let d011 = gradient_dot(
        improved_noise_permutation(snapshot, xy01 + zf + 1),
        xr,
        yr - yr_fudge - 1.0,
        zr - 1.0,
    );
    let d111 = gradient_dot(
        improved_noise_permutation(snapshot, xy11 + zf + 1),
        xr - 1.0,
        yr - yr_fudge - 1.0,
        zr - 1.0,
    );

    lerp3(
        smoothstep(xr),
        smoothstep(yr),
        smoothstep(zr),
        d000,
        d100,
        d010,
        d110,
        d001,
        d101,
        d011,
        d111,
    )
}

struct ImprovedNoiseDerivativeCell {
    fractional: [f64; 3],
    alphas: [f64; 3],
    gradients: [[i32; 3]; 8],
    dot_values: [f64; 8],
}

fn improved_noise_derivative_cell(
    snapshot: &ImprovedNoiseSnapshot,
    input_x: f64,
    input_y: f64,
    input_z: f64,
) -> ImprovedNoiseDerivativeCell {
    let x = input_x + snapshot.xo;
    let y = input_y + snapshot.yo;
    let z = input_z + snapshot.zo;
    let xf = x.floor() as i32;
    let yf = y.floor() as i32;
    let zf = z.floor() as i32;
    let xr = x - f64::from(xf);
    let yr = y - f64::from(yf);
    let zr = z - f64::from(zf);
    let hashes = improved_noise_corner_hashes(snapshot, xf, yf, zf);
    let gradients = hashes.map(simplex_gradient);
    let dot_values = [
        simplex_dot(gradients[0], xr, yr, zr),
        simplex_dot(gradients[1], xr - 1.0, yr, zr),
        simplex_dot(gradients[2], xr, yr - 1.0, zr),
        simplex_dot(gradients[3], xr - 1.0, yr - 1.0, zr),
        simplex_dot(gradients[4], xr, yr, zr - 1.0),
        simplex_dot(gradients[5], xr - 1.0, yr, zr - 1.0),
        simplex_dot(gradients[6], xr, yr - 1.0, zr - 1.0),
        simplex_dot(gradients[7], xr - 1.0, yr - 1.0, zr - 1.0),
    ];

    ImprovedNoiseDerivativeCell {
        fractional: [xr, yr, zr],
        alphas: [smoothstep(xr), smoothstep(yr), smoothstep(zr)],
        gradients,
        dot_values,
    }
}

fn gradient_component_values(gradients: [[i32; 3]; 8], component: usize) -> [f64; 8] {
    gradients.map(|gradient| f64::from(gradient[component]))
}

fn derivative_difference_terms(alphas: [f64; 3], dot_values: [f64; 8]) -> [f64; 3] {
    [
        lerp2(
            alphas[1],
            alphas[2],
            dot_values[1] - dot_values[0],
            dot_values[3] - dot_values[2],
            dot_values[5] - dot_values[4],
            dot_values[7] - dot_values[6],
        ),
        lerp2(
            alphas[2],
            alphas[0],
            dot_values[2] - dot_values[0],
            dot_values[6] - dot_values[4],
            dot_values[3] - dot_values[1],
            dot_values[7] - dot_values[5],
        ),
        lerp2(
            alphas[0],
            alphas[1],
            dot_values[4] - dot_values[0],
            dot_values[5] - dot_values[1],
            dot_values[6] - dot_values[2],
            dot_values[7] - dot_values[3],
        ),
    ]
}

pub fn improved_noise_sample_with_derivative(
    snapshot: &ImprovedNoiseSnapshot,
    input_x: f64,
    input_y: f64,
    input_z: f64,
    derivative_out: &mut [f64; 3],
) -> f64 {
    let cell = improved_noise_derivative_cell(snapshot, input_x, input_y, input_z);
    let gradient_terms = [
        lerp3_corners(cell.alphas, gradient_component_values(cell.gradients, 0)),
        lerp3_corners(cell.alphas, gradient_component_values(cell.gradients, 1)),
        lerp3_corners(cell.alphas, gradient_component_values(cell.gradients, 2)),
    ];
    let difference_terms = derivative_difference_terms(cell.alphas, cell.dot_values);
    for axis in 0..3 {
        derivative_out[axis] += gradient_terms[axis]
            + smoothstep_derivative(cell.fractional[axis]) * difference_terms[axis];
    }
    lerp3_corners(cell.alphas, cell.dot_values)
}
