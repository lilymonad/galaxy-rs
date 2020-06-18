use las::{Writer, Write, Color, Builder, Transform, Vector, Point as LPoint};
use std::{io::BufWriter, fs::OpenOptions};
use rand::prelude::*;

mod point;
use point::Point;


const NB_LONERS : u64 = 16;

fn populate_frame(position:Point, nb_arms:u32, vec:&mut Vec<LPoint>) {
    // push root
    vec.push(position.with_color(Color::new(u16::MAX, 0, 0)));

    // populate arms
    let angle_step = (2f64 * std::f64::consts::PI) / (nb_arms as f64);
    let mut arm_angle = 0f64;
    for _ in 0..nb_arms {
        populate_arm(position, arm_angle, vec);
        arm_angle += angle_step;
    }

    // get galaxy radius == max of distances from the root
    let galaxy_radius = vec.iter().map(|p| p.x*p.x + p.y*p.y).fold(0.0, |a:f64,b| a.max(b)).sqrt();

    // populate loners
    for _ in 0..NB_LONERS {
        let angle = rand::thread_rng().gen_range(0f64, 2.0 * std::f64::consts::PI);
        let dist = rand::thread_rng().gen_range(0f64, galaxy_radius);

        let loner_point = Point::polar(dist, angle);
        vec.push(loner_point.with_color(Color::new(0, 0, u16::MAX)))
    }
}

const SLOPE : f64 = std::f64::consts::PI / 16f64;
const ARM_POINTS : u64 = 8;
const ARM_POINT_DISTANCE : f64 = 16f64;

fn populate_arm(mut position:Point, mut arm_angle:f64, vec:&mut Vec<LPoint>) {
    for i in 1..=ARM_POINTS {
        let new_position = position + Point::polar(ARM_POINT_DISTANCE, arm_angle);

        vec.push(new_position.with_color(Color::new(u16::MAX, 0, 0)));
        populate_ext(new_position, i, (new_position - position).normalize(), vec);

        position = new_position;
        arm_angle += SLOPE;
    }
}

fn populate_ext(position:Point, iteration:u64, direction:Point, vec: &mut Vec<LPoint>) {
    // take a 2D normal of our direction vector
    let normale = direction.minusb_a() * iteration as f64 * ARM_POINT_DISTANCE / 8f64;
    let color = Color::new(u16::MAX, u16::MAX, 0);

    // add 2 arms, at +normal and -normal offset from the starting point
    vec.push((position + normale).with_color(color));
    vec.push((position - normale).with_color(color))
}

const SYSTEM_CLOUD_RADIUS : f64 = 8f64;

fn populate_systems(vec:Vec<LPoint>) -> Vec<LPoint> {
    vec.into_iter().flat_map(|p| {
        (0..=8).into_iter().map(move |i| {
            let p = p.clone();

            let angle = rand::thread_rng().gen_range(0.0, 2.0 * std::f64::consts::PI);
            let radius = rand::thread_rng().gen_range(0.0, SYSTEM_CLOUD_RADIUS);

            // 0 value is used to keep the generating point
            if i == 0 {
                p
            } else {
                let np : Point = Point::from(p) + Point::polar(radius, angle);
                np.with_color(Color::new(u16::MAX, 0, u16::MAX))
            }
        })
    })
    .collect()
}

fn main() {
    // create the skeleton points
    let mut vec = Vec::new();
    populate_frame(Point { x:0f64, y:0f64 }, 3, &mut vec);

    // add systems as little point clouds near every skeleton point
    let vec = populate_systems(vec);

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

    // write points into the file
    let mut las_writer = Writer::new(buf_file, las_header).expect("Cannot create LAS writer");
    for p in vec {
        let _ = las_writer.write(p);
    }
}
