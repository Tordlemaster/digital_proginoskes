use std::str::FromStr;

const MAX_DEPTH: i32 = 4;


pub struct StarData {
    pub ra: f32, //Right ascension
    pub dec: f32, //Declination
    pub bt: f32, 
    pub vt: f32
}

pub struct SphQtNode {
    pub stars: Vec<StarData>,
    pub corners: [[f32; 2]; 2], //corners, least coordinates in position 0 and greatest coordinates in position 1
    pub midpoint: [f32; 2],
    axes: [usize; 2], //x, y, z
    inactive_ax: usize,
    pub children: [Option<Box<SphQtNode>>; 4],
    pub stars_in_children: u64
}


pub struct SphQtRoot {
    pub faces: [Option<Box<SphQtNode>>; 6], //N, S, W, E, T, B
    pub star_count: usize
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseStarDataError;
impl FromStr for StarData {
    type Err = ParseStarDataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut x = s.split_whitespace().map(|s| s.parse::<f32>().map_err(|_| ParseStarDataError));
        let ra = x.next().ok_or(ParseStarDataError)??;
        let dec = x.next().ok_or(ParseStarDataError)??;
        let bt = x.next().ok_or(ParseStarDataError)??;
        let vt = x.next().ok_or(ParseStarDataError)??;

        Ok(StarData { ra: ra, dec: dec, bt: bt, vt: vt })
    }
}

impl SphQtNode {
    pub fn new(corners: [[f32; 2]; 2], ax: [usize; 2], inac_ax: usize) -> SphQtNode {
        //Switch bl and tr if tr has lower coordinates
        let lowest_coords: [usize; 2] = [(corners[1][0] > corners[0][0]).into(), (corners[1][1] > corners[0][1]).into()];

        SphQtNode {
            stars: Vec::new(),
            corners: [
                [corners[(lowest_coords[0] != 1) as usize][0], corners[(lowest_coords[1] != 1) as usize][1]],
                [corners[lowest_coords[0]][0], corners[lowest_coords[1]][1]],
                
            ],
            midpoint: [(corners[0][0] + corners[1][0]) / 2.0, (corners[0][1] + corners[1][1]) / 2.0],
            axes: ax,
            inactive_ax: inac_ax,
            children: [const{None}; 4],
            stars_in_children: 0
        }
    }
}

impl SphQtRoot {
    pub fn new() -> SphQtRoot {
        //North: 90, 45; 0; -45 
        //South: 270, 45; 180, -45
        //West: 360, 45; 270, -45
        //East: 180, 45; 90, -45
        //Top: 180, 45; 0, 45
        //Bottom: 90, -45; 270, -45
        SphQtRoot { faces: [
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [1, 2], 0))), //X+
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [1, 2], 0))), //X-
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [0, 2], 1))), //Y+
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [0, 2], 1))), //Y-
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [0, 1], 2))), //Z+
            Some(Box::new(SphQtNode::new([[-1.0, -1.0], [1.0, 1.0]], [0, 1], 2))), //Z-
        ], star_count: 0}
    }

    pub fn add(&mut self, star: StarData) {
        //Determine face of cube sphere TODO REWORK
        /*let mut face_idx = 0;
        let mut axis: u8 = 0;
        if star.dec > 45.0 { //Top
            face_idx = 0;
            axis = 2;
        }
        else if star.dec < -45.0 { //Bottom
            face_idx = 1;
            axis = 2;
        }
        else if star.ra > 315.0 && star.ra <= 45.0 { //North
            face_idx = 2;
            axis = 0;
        }
        else if star.ra > 135.0 && star.ra <= 225.0 { //South
            face_idx = 3;
            axis = 0;
        }
        else if star.ra > 45.0 && star.ra <= 135.0 { //East
            face_idx = 4;
            axis = 1;
        }
        else { //West
            face_idx = 5;
            axis = 1;
        }*/

        //Convert coordinates to Cartesian
        let star_pos: [f32; 3] = [
            star.dec.cos() * star.ra.cos(),
            star.dec.cos() * star.ra.sin(),
            star.dec.sin()
        ];

        //Determine face of cube sphere
        let mut greatest_ax_val = star_pos[0].abs(); //X axis
        let mut face_idx: usize = (star_pos[0] >= 0.0).into();
        let mut axes: [usize; 2] = [1, 2];
        let mut inactive_ax = 0;

        if star_pos[1].abs() > greatest_ax_val { //Y axis
            greatest_ax_val = star_pos[1].abs();
            face_idx = (star_pos[1] >= 0.0) as usize + 2;
            axes = [0, 2];
            inactive_ax = 1;
        }
        if star_pos[2].abs() > greatest_ax_val { //Z axis
            greatest_ax_val = star_pos[2].abs();
            face_idx = (star_pos[2] >= 0.0) as usize + 4;
            axes = [0, 1];
            inactive_ax = 2;
        }

        let star_pos_2d = [star_pos[axes[0]], star_pos[axes[1]]];
        
        //recur downward through the quadtree of that face
        let mut depth = 0;
        let mut cur_parent = &mut self.faces[face_idx];
        while depth < MAX_DEPTH {
            let uw_parent = cur_parent.as_mut().unwrap();
            //determine cell
            let child_idx: usize = (((star_pos_2d[1] > uw_parent.midpoint[1]) as usize) << 1) | (star_pos_2d[0] > uw_parent.midpoint[0]) as usize;
            assert!(child_idx < 4);

            //child_idx selects the corner of the cell to use as the corner of the new cell along with the midpoint

            //Instantiate the child
            if uw_parent.children[child_idx].is_none() {
                //HOW TO ELIMINATE THE UNUSED AXIS FROM THE STORAGE FOR SPHQTNODE
                //The other corner is { corners[child_idx[x]][0], corners[child_idx[y]][1] }
                uw_parent.children[child_idx] = Some(Box::new(SphQtNode::new([uw_parent.midpoint, [uw_parent.corners[child_idx & 1][0], uw_parent.corners[child_idx >> 1][1]]], uw_parent.axes, uw_parent.inactive_ax)));
            }

            //Add the child's star to the parent's count
            uw_parent.stars_in_children += 1;

            //Recur down a layer
            cur_parent = &mut uw_parent.children[child_idx];

            depth += 1;
        }
        //Add star data to the leaf node
        cur_parent.as_mut().unwrap().stars.push(star);
        cur_parent.as_mut().unwrap().stars_in_children += 1;
        self.star_count += 1;

        //append idx to the star_idxs at the leaf node
    }
}