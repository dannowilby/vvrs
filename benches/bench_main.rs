use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::chunk_meshing::benches,
    benchmarks::vis_graph_creation::benches
}
