use criterion::{black_box, criterion_group, criterion_main, Criterion};

const N: usize = 960;

fn convert_f32_to_u32_standard(x: &[f32; N]) -> [u32; N] {
    let mut y = [0u32; N];
    for i in 0..N {
        y[i] = (((x[i] + 1.0) / 2.0) * (u32::MAX as f32)) as u32;
    }
    y
}

fn convert_f32_to_i32_swyh(x: &[f32; N]) -> [i32; N] {
    let mut y = [0i32; N];
    for i in 0..N {
        let sample = x[i].clamp(-1.0, 1.0);
        y[i] = if sample >= 0.0 {
            ((sample as f64 * i32::MAX as f64) + 0.5) as i32
        } else {
            ((-sample as f64 * i32::MIN as f64) - 0.5) as i32
        };
    }
    y
}


#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn convert_f32_to_u32_simd(x: &[f32; N]) -> [u32; N] {
    let mut y = [0u32; N];
    unsafe {
        for i in (0..N).step_by(8) {
            let x_simd = _mm256_loadu_ps(&x[i] as *const f32);
            let one = _mm256_set1_ps(1.0);
            let two = _mm256_set1_ps(2.0);
            let max = _mm256_set1_ps(u32::MAX as f32);
            let y_simd = _mm256_mul_ps(_mm256_div_ps(_mm256_add_ps(x_simd, one), two), max);
            let y_ptr = &mut y[i] as *mut u32 as *mut f32;
            _mm256_storeu_ps(y_ptr, y_simd);
        }
    }
    y
}

fn convert_f32_to_i32_simd(x: &[f32; N]) -> [i32; N] {
    let mut y = [0i32; N];
    unsafe {
        let one = _mm256_set1_ps(1.0);
        let max = _mm256_set1_ps(i32::MAX as f32);
        let min = _mm256_set1_ps(i32::MIN as f32);
        let half = _mm256_set1_ps(0.5);
        for i in (0..N).step_by(8) {
            let x_simd = _mm256_loadu_ps(&x[i] as *const f32);
            let x_clamped = _mm256_max_ps(_mm256_min_ps(x_simd, one), _mm256_sub_ps(_mm256_setzero_ps(), one));
            let mask = _mm256_cmp_ps(x_clamped, _mm256_setzero_ps(), _CMP_GE_OS);
            let y_positive = _mm256_cvttps_epi32(_mm256_add_ps(_mm256_mul_ps(x_clamped, max), half));
            let y_negative = _mm256_cvttps_epi32(_mm256_sub_ps(_mm256_mul_ps(_mm256_sub_ps(_mm256_setzero_ps(), x_clamped), min), half));
            let y_simd = _mm256_blendv_epi8(y_negative, y_positive, _mm256_castps_si256(mask));
            _mm256_storeu_si256(&mut y[i] as *mut i32 as *mut __m256i, y_simd);
        }
    }
    y
}

fn convert_f32_to_i32_swyh_simple(x: &[f32; N]) -> [i32; N] {
    let mut y = [0i32; N];
    for i in 0..N {
        let sample = x[i].clamp(-1.0, 1.0);
        y[i] = if sample >= 0.0 {
            (sample * i32::MAX as f32 + 0.5) as i32
        } else {
            (-sample * i32::MIN as f32 - 0.5) as i32
        };
    }
    y
}

fn criterion_benchmark(c: &mut Criterion) {
    let x: [f32; N] = [0.5; N];

    c.bench_function("standard (u32)", |b| b.iter(|| convert_f32_to_u32_standard(black_box(&x))));
    c.bench_function("simd (u32)", |b| b.iter(|| convert_f32_to_u32_simd(black_box(&x))));
    c.bench_function("standard (i32)", |b| b.iter(|| convert_f32_to_i32_swyh(black_box(&x))));
    c.bench_function("simd (i32)", |b| b.iter(|| convert_f32_to_i32_simd(black_box(&x))));
    c.bench_function("standard (i32) simple", |b| b.iter(|| convert_f32_to_i32_swyh_simple(black_box(&x))));

    // result shows that the SIMD versions are a lot slower than the standard versions
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);