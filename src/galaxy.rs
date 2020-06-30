use petgraph::{Graph, graph::{NodeIndex, NodeIndices}, visit::EdgeRef};
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EdgeType {
    Frame,
    Overlapping,
}

pub struct GalaxyBuilder {
    nb_loners : u64,
    arm_slope : f64,
    cloud_radius : f64,
    cloud_population : u64,
    nb_arms : u64,
    arm_width_factor : f64,
    nb_arm_bones : u64,
    slope_factor : f64,
    min_distance : Option<f64>,
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
    setter!{min_distance : Option<f64>}

    fn add_node(&self, weight:DataPoint<NodeType>, graph:&mut Graph<DataPoint<NodeType>, EdgeType>) -> NodeIndex<u32> {
        let idx = graph.add_node(weight);
        for idy in graph.node_indices() {
            let p2 = &graph[idy];
            if (weight.point - p2.point).length() < self.cloud_radius * 2.0 {
                graph.add_edge(idx, idy, EdgeType::Overlapping);
            }
        }
        idx
    }

    pub fn build(&self, center:Point) -> Option<Galaxy> {

        if let Some(dist) = self.min_distance {
            if dist > self.cloud_radius { return None }
        }

        let nb_frame_points = 1 + 3 * self.nb_arms * self.nb_arm_bones + self.nb_loners;
        let estimate_nb_nodes = (1 + self.cloud_population) * nb_frame_points;
        let mut graph = Graph::new();
        graph.reserve_nodes(estimate_nb_nodes as usize);
        graph.reserve_edges(estimate_nb_nodes as usize);

        // push root
        let root_id = self.add_node(DataPoint::from_point(center, Root), &mut graph);

        // populate arms
        let angle_step = (2.0 * std::f64::consts::PI) / (self.nb_arms as f64);
        let mut arm_angle = 0f64;
        for _ in 0..self.nb_arms {
            self.populate_arm(root_id, arm_angle, &mut graph);
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
            self.add_node(loner_point, &mut graph);
        }

        self.populate_cloud(&mut graph);
        Some(Galaxy {
            graph : graph.filter_map(
                        |_, &n| {
                            Some(n)
                        },
                        |_, e| {
                            if e == &EdgeType::Frame { Some(()) } else { None }
                        })
        })
    }

    fn populate_arm(&self, mut root_id:NodeIndex<u32>, mut arm_angle:f64, frame:&mut Graph<DataPoint<NodeType>, EdgeType>) {
        
        let mut position = frame[root_id].point;
        let mut divisor = 1.0;
        let bone_length = self.cloud_radius;

        for iteration in 0..self.nb_arm_bones {
            let new_position = position + Point::polar(bone_length, arm_angle);

            let arm_id = self.add_node(new_position.with_data(Arm), frame);
            frame.add_edge(root_id, arm_id, EdgeType::Frame);

            if iteration != 0 {
                self.populate_ext(arm_id, iteration, (new_position - position).normalize(), frame);
            }

            root_id = arm_id;
            position = new_position;
            arm_angle += self.arm_slope / divisor;
            divisor += self.slope_factor;
        }
    }

    fn populate_ext(&self, arm_id:NodeIndex<u32>, iteration:u64, direction:Point, frame: &mut Graph<DataPoint<NodeType>, EdgeType>) {
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
            let ext1 = self.add_node((position + normale * n).with_data(Ext), frame);
            let ext2 = self.add_node((position - normale * n).with_data(Ext), frame);
            frame.add_edge(prev1.unwrap_or(arm_id), ext1, EdgeType::Frame);
            frame.add_edge(prev2.unwrap_or(arm_id), ext2, EdgeType::Frame);
            prev1 = Some(ext1);
            prev2 = Some(ext2);
        }
    }

    fn populate_cloud(&self, frame:&mut Graph<DataPoint<NodeType>, EdgeType>) {

        let min_distance = self.min_distance.unwrap_or(0.0);
        for i in frame.node_indices() {

            let p = frame[i].point;

            'generate: for _ in 0..self.cloud_population {
                let angle = thread_rng().gen_range(0.0, 2.0 * std::f64::consts::PI);
                let dist = thread_rng().gen_range(min_distance, self.cloud_radius);
                let np = p + Point::polar(dist, angle);

                if self.min_distance.is_some() {
                    for edge in frame.edges(i).filter(|e| e.weight() == &EdgeType::Overlapping) {
                        let (id1, id2) = frame.edge_endpoints(edge.id()).unwrap();
                        let id = if id1 == i { id2 } else { id1 };

                        for edge in frame.edges(id).filter(|e| e.weight() == &EdgeType::Frame) {
                            let (id1, id2) = frame.edge_endpoints(edge.id()).unwrap();
                            let id = if id1 == id { id2 } else { id1 };
                            let point = frame[id].point;

                            let distance = (point - np).length();
                            if distance < min_distance {
                                continue 'generate
                            }
                        }
                    }
                }

                let sys_id = self.add_node(np.with_data(System), frame);
                frame.add_edge(i, sys_id, EdgeType::Frame);
            }
        }
    }
}

/// Default config because why not
impl Default for GalaxyBuilder {
    fn default() -> Self {
        Self {
            min_distance : None,
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

pub struct Galaxy {
    graph : Graph<DataPoint<NodeType>, ()>,
}

impl Galaxy {
    pub fn into_inner(self) -> Graph<DataPoint<NodeType>, ()> {
        self.graph
    }

    pub fn into_mapped<F, E>(self, f:F) -> Graph<E, ()>
        where F : Fn(&DataPoint<NodeType>) -> E,
    {
        self.graph.map(|_, dp| f(dp), |_, _| ())
    }
}

impl IntoIterator for Galaxy {
    type Item = DataPoint<NodeType>;
    type IntoIter = GalaxyPoints;

    fn into_iter(self) -> GalaxyPoints {
        GalaxyPoints::new(self.graph)
    }
}

pub struct GalaxyPoints {
    graph  : Graph<DataPoint<NodeType>, ()>,
    points : NodeIndices<u32>,
}

impl GalaxyPoints {
    fn new(graph:Graph<DataPoint<NodeType>, ()>) -> Self {
        Self {
            points : graph.node_indices(),
            graph,
        }
    }
}

impl Iterator for GalaxyPoints {
    type Item = DataPoint<NodeType>;

    fn next(&mut self) -> Option<DataPoint<NodeType>> {
        match self.points.next() {
            Some(idx) => Some(self.graph[idx]),
            None => None,
        }
    }
}
