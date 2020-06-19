use las::{Writer, Write, Color, Builder, Transform, Vector};
use std::{io::BufWriter, fs::OpenOptions};
use rand::prelude::*;
use petgraph::{Graph, graph::NodeIndex};

mod point;
mod frame;
use point::{Point, DataPoint};
use frame::NodeType::{self, *};

const NB_LONERS : u64 = 16;

fn populate_frame(center:Point, nb_arms:u32, frame:&mut Graph<DataPoint<NodeType>, ()>) {
    // push root
    let root_id = frame.add_node(DataPoint::from_point(center, Root));

    // populate arms
    let angle_step = (2f64 * std::f64::consts::PI) / (nb_arms as f64);
    let mut arm_angle = 0f64;
    for _ in 0..nb_arms {
        populate_arm(root_id, arm_angle, frame);
        arm_angle += angle_step;
    }

    // get galaxy radius == max of distances from the root
    let galaxy_radius = frame.node_indices()
        .map(|ip| {
            let p = frame[ip].point;
            p.x*p.x + p.y*p.y
        })
        .fold(0.0, |a:f64,b| a.max(b))
        .sqrt();

    // populate loners
    for _ in 0..NB_LONERS {
        let angle = thread_rng().gen_range(0f64, 2.0 * std::f64::consts::PI);
        let dist = thread_rng().gen_range(0f64, galaxy_radius);

        let loner_point = DataPoint::polar(dist, angle, Loner);
        frame.add_node(loner_point);
    }

    populate_cloud(frame)
}

const SLOPE : f64 = std::f64::consts::PI / 8f64;
const ARM_POINTS : u64 = 8;
const ARM_BONE_LENGTH : f64 = 16f64;

fn populate_arm(mut root_id:NodeIndex<u32>, mut arm_angle:f64, frame:&mut Graph<DataPoint<NodeType>, ()>) {
    
    let mut position = frame[root_id].point;

    for i in 1..=ARM_POINTS {
        let new_position = position + Point::polar(ARM_BONE_LENGTH, arm_angle);

        let arm_id = frame.add_node(new_position.with_data(Arm));
        frame.add_edge(root_id, arm_id, ());
        populate_ext(arm_id, i, (new_position - position).normalize(), frame);

        position = new_position;
        arm_angle += SLOPE;
        root_id = arm_id;
    }
}

fn populate_ext(arm_id:NodeIndex<u32>, iteration:u64, direction:Point, frame: &mut Graph<DataPoint<NodeType>, ()>) {
    let position = frame[arm_id].point;

    // take a 2D normal of our direction vector
    let normale = direction.minusb_a() * iteration as f64 * ARM_BONE_LENGTH / 8f64;

    // add 2 arms, at +normal and -normal offset from the starting point
    let ext1 = frame.add_node((position + normale).with_data(Ext));
    let ext2 = frame.add_node((position - normale).with_data(Ext));
    frame.add_edge(arm_id, ext1, ());
    frame.add_edge(arm_id, ext2, ());
}

const SYSTEM_CLOUD_RADIUS : f64 = ARM_BONE_LENGTH;
const SYSTEM_CLOUD_POPULATION : u64 = 2;

fn populate_cloud(frame:&mut Graph<DataPoint<NodeType>, ()>) {
    for i in frame.node_indices() {

        let p = frame[i].point;

        for _ in 0..SYSTEM_CLOUD_POPULATION {
            let angle = thread_rng().gen_range(0.0, 2.0 * std::f64::consts::PI);
            let dist = thread_rng().gen_range(0.0, SYSTEM_CLOUD_RADIUS);

            let np = Point::polar(dist, angle) + p;


            let sys_id = frame.add_node(np.with_data(System));
            frame.add_edge(i, sys_id, ());
        }
    }
}

fn main() {
    // create the galaxy graph
    let mut frame = Graph::new();
    populate_frame(Point { x:0f64, y:0f64 }, 5, &mut frame);

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

        let _ = las_writer.write(p.map(Into::<Color>::into).to_lidar_with_z(z));
    }
}
