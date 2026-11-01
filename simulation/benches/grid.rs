use criterion::{criterion_group, criterion_main, Criterion};
use tellus_sim::shared::grid::Grid;
use tellus_sim::shared::units::R_EARTH_M;

fn bench_grid(c: &mut Criterion) {
    c.bench_function("grid_build_l7", |b| b.iter(|| Grid::new(7, R_EARTH_M)));
    let grid = Grid::new(7, R_EARTH_M);
    c.bench_function("texel_lookup_l7_2048x1024", |b| {
        b.iter(|| grid.texel_lookup(2048, 1024))
    });
}

criterion_group!(benches, bench_grid);
criterion_main!(benches);
