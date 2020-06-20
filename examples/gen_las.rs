///
///
///run with: cargo run --example las_gen --features lidar
///
///
use las_lib::{Writer, Write, Color, Builder, Transform, Vector};
use std::{io::BufWriter, fs::OpenOptions};
use rand::prelude::*;
use petgraph::Graph;
use galaxy_rs::{GalaxyBuilder, Point};

fn main() {
    // create the galaxy graph
    let mut frame = Graph::new();
    GalaxyBuilder::default()
        .cloud_population(32)
        .nb_arms(5)
        .nb_arm_bones(32)
        .slope_factor(0.4)
        .arm_slope(std::f64::consts::PI / 4.0)
        .arm_width_factor(1.0 / 24.0)
        .populate(Point { x:0f64, y:0f64 }, &mut frame);

    // open the result file
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("galaxy.las")
        .expect("Cannot open file galaxy.las");
    let buf_file = BufWriter::new(file);

    // define the LAS format header 
    let mut las_hd_builder = Builder::new(Default::default()).unwrap();
    las_hd_builder.point_format.has_color = true;
    las_hd_builder.transforms = Vector{
        x: Transform { scale: 1f64, offset: 0f64 },
        y: Transform { scale: 1f64, offset: 0f64 },
        z: Transform { scale: 1f64, offset: 0f64 },
    };
    let las_header = las_hd_builder.into_header().unwrap();

    // write points from the graph into the file
    let mut las_writer = Writer::new(buf_file, las_header).expect("Cannot create LAS writer");
    for p in frame.node_indices().map(|id| frame[id]) {
        // 2000.0 is a self-tuned constant
        // TODO: define the divisor from parameters like galaxy radius
        let np = p.point;
        let gauss = (-np.dot(np) / 2000.0).exp();
        let z = thread_rng().gen_range(-16.0, 16.0) * gauss;

        las_writer.write(p.map(Into::<Color>::into).to_lidar_with_z(z)).unwrap();
    }
}
