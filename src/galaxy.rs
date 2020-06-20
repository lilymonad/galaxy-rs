use petgraph::{Graph, graph::NodeIndex};
use crate::point::{Point, DataPoint};
use rand::prelude::*;

#[derive(Clone, Copy)]
pub enum NodeType {
    Root,
    Arm,
    Ext,
    Loner,
    System,
}

use NodeType::*;

pub struct GalaxyBuilder {
    nb_loners : u64,
    arm_slope : f64,
    cloud_radius : f64,
    cloud_population : u64,
    nb_arms : u64,
    arm_width_factor : f64,
    nb_arm_bones : u64,
    slope_factor : f64,
}

macro_rules! setter {
    { $name:ident : $type:ty } => {
        pub fn $name (self, $name:$type) -> Self {
            Self {
                $name,
                ..self
            }
        }
    }
}

impl GalaxyBuilder {

    setter!{nb_arm_bones : u64}
    setter!{slope_factor : f64}
    setter!{arm_width_factor : f64}
    setter!{nb_arms : u64}
    setter!{nb_loners : u64}
    setter!{arm_slope : f64}
    setter!{cloud_radius : f64}
    setter!{cloud_population : u64}

    pub fn populate(&self, center:Point, graph:&mut Graph<DataPoint<NodeType>, ()>) {
        // push root
        let root_id = graph.add_node(DataPoint::from_point(center, Root));

        // populate arms
        let angle_step = (2.0 * std::f64::consts::PI) / (self.nb_arms as f64);
        let mut arm_angle = 0f64;
        for _ in 0..self.nb_arms {
            self.populate_arm(root_id, arm_angle, graph);
            arm_angle += angle_step;
        }

        // get galaxy radius == max of distances from the root
        let galaxy_radius = graph.node_indices()
            .map(|ip| {
                let p = graph[ip].point;
                p * p
            })
            .fold(0.0, |a:f64,b| a.max(b))
            .sqrt();

        // populate loners
        for _ in 0..self.nb_loners {
            let angle = thread_rng().gen_range(0f64, 2.0 * std::f64::consts::PI);
            let dist = thread_rng().gen_range(0f64, galaxy_radius);

            let loner_point = DataPoint::polar(dist, angle, Loner);
            graph.add_node(loner_point);
        }

        self.populate_cloud(graph)
    }

    fn populate_arm(&self, mut root_id:NodeIndex<u32>, mut arm_angle:f64, frame:&mut Graph<DataPoint<NodeType>, ()>) {
        
        let mut position = frame[root_id].point;
        let mut divisor = 1.0;
        let bone_length = self.cloud_radius;

        for iteration in 0..self.nb_arm_bones {
            let new_position = position + Point::polar(bone_length, arm_angle);

            let arm_id = frame.add_node(new_position.with_data(Arm));
            frame.add_edge(root_id, arm_id, ());

            if iteration != 0 {
                self.populate_ext(arm_id, iteration, (new_position - position).normalize(), frame);
            }

            root_id = arm_id;
            position = new_position;
            arm_angle += self.arm_slope / divisor;
            divisor += self.slope_factor;
        }
    }

    fn populate_ext(&self, arm_id:NodeIndex<u32>, iteration:u64, direction:Point, frame: &mut Graph<DataPoint<NodeType>, ()>) {
        let position = frame[arm_id].point;
        let bone_length = self.cloud_radius * 2.0;

        // take a 2D normal of our direction vector
        let mut normale = direction.minusb_a() * iteration as f64 * bone_length * self.arm_width_factor;

        let length = normale.length();
        let nb_points = ((length / bone_length).floor() as u32) + 1;
        normale = normale / nb_points as f64;

        let (mut prev1, mut prev2) = (None, None);
        for i in 1..=nb_points {
            let n = i as f64;
            // add 2 arms, at +normal and -normal offset from the starting point
            let ext1 = frame.add_node((position + normale * n).with_data(Ext));
            let ext2 = frame.add_node((position - normale * n).with_data(Ext));
            frame.add_edge(prev1.unwrap_or(arm_id), ext1, ());
            frame.add_edge(prev2.unwrap_or(arm_id), ext2, ());
            prev1 = Some(ext1);
            prev2 = Some(ext2);
        }
    }

    fn populate_cloud(&self, frame:&mut Graph<DataPoint<NodeType>, ()>) {
        for i in frame.node_indices() {

            let p = frame[i].point;

            for _ in 0..self.cloud_population {
                let angle = thread_rng().gen_range(0.0, 2.0 * std::f64::consts::PI);
                let dist = thread_rng().gen_range(0.0, self.cloud_radius);

                let np = p + Point::polar(dist, angle);

                let sys_id = frame.add_node(np.with_data(System));
                frame.add_edge(i, sys_id, ());
            }
        }
    }
}

impl Default for GalaxyBuilder {
    fn default() -> Self {
        Self {
            arm_width_factor : 1.0 / 16.0,
            nb_arms : 5,
            nb_loners : 16,
            arm_slope : std::f64::consts::PI / 8.0,
            nb_arm_bones : 12,
            cloud_radius : 16.0,
            cloud_population : 2,
            slope_factor : 0.90,
        }
    }
}
