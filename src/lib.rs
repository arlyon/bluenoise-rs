use glam::Vec2;
use itertools::Itertools;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

#[derive(Debug)]
struct BlueNoise {
    width: u32,
    height: u32,
    max_samples: u32,

    /// The minimum radius between points
    radius: f32,
    radius_squared: f32,

    cell_size: f32,
    grid: Vec<Option<Vec2>>,
    grid_width: usize,
    grid_height: usize,

    /// A list of points that we can generate new
    /// points around.
    active_points: Vec<Vec2>,

    rng: Pcg64Mcg,
    init: bool,
}

impl BlueNoise {
    pub fn new(width: u32, height: u32, radius: f32) -> Self {
        let cell_size = radius * FRAC_1_SQRT_2;
        let grid_width = (width as f32 / cell_size).ceil() as usize;
        let grid_height = (height as f32 / cell_size).ceil() as usize;
        let grid = vec![None; grid_width * grid_height];
        let radius_squared = radius * radius;
        let rng = SeedableRng::from_entropy();

        Self {
            width,
            height,
            max_samples: 4,
            radius,
            radius_squared,
            cell_size,
            grid,
            grid_width,
            grid_height,
            active_points: Default::default(),
            rng,
            init: false,
        }
    }

    pub fn with_samples(&mut self, max_samples: u32) -> &mut Self {
        self.max_samples = max_samples;
        self
    }

    pub fn with_seed(&mut self, seed: u64) -> &mut Self {
        self.rng = SeedableRng::seed_from_u64(32);
        self
    }

    /// Check if a position is far enough away from
    /// nearby previously created points.
    fn is_valid(&self, point: Vec2) -> bool {
        // remove anything outside our box
        if point.x() < 0.0
            || point.x() > self.width as f32
            || point.y() < 0.0
            || point.y() > self.height as f32
        {
            return false;
        };

        let x_range = {
            let x = (point.x() / self.cell_size) as usize;
            x.saturating_sub(2)..(x + 3).min(self.grid_width as usize)
        };

        let y_range = {
            let y = (point.y() / self.cell_size) as usize;
            y.saturating_sub(2)..(y + 3).min(self.grid_height as usize)
        };

        x_range.cartesian_product(y_range).all(|(x, y)| {
            // if there is a point, check if it is furthern than our min radius
            match self.grid[y * self.grid_width + x] {
                Some(target) => (target - point).length_squared() > self.radius_squared,
                None => true,
            }
        })
    }

    /// Get the index for a given position
    fn grid_index(&self, position: Vec2) -> usize {
        let y = self.grid_width * (position.y() / self.cell_size) as usize;
        let x = (position.x() / self.cell_size) as usize;
        let out = y + x;

        assert_ne!(self.grid_width * self.grid_height, x);

        out
    }

    /// Insert a point into the grid and mark it active
    fn insert_point(&mut self, position: Vec2) -> Vec2 {
        let index = self.grid_index(position);
        self.grid[index] = Some(position);
        self.active_points.push(position);
        position
    }

    /// Get some nearby point
    fn get_nearby(&mut self, position: Vec2) -> Vec2 {
        let theta = 2.0 * PI * self.rng.gen::<f32>();
        let radius = self.radius * (self.rng.gen::<f32>() + 1.0);
        Vec2::new(
            position.x() + radius * theta.cos(),
            position.y() + radius * theta.sin(),
        )
    }
}

impl Iterator for BlueNoise {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            self.init = true;
            let x = self.rng.gen_range(0.0, self.width as f32);
            let y = self.rng.gen_range(0.0, self.height as f32);
            return Some(self.insert_point(Vec2::new(x, y)));
        }

        while !self.active_points.is_empty() {
            let index = self.rng.gen::<f32>() * (self.active_points.len() - 1) as f32;
            let parent = self.active_points[index as usize];

            for _ in 0..self.max_samples {
                let point = self.get_nearby(parent);
                if self.is_valid(point) {
                    return Some(self.insert_point(point));
                }
            }

            self.active_points.remove(index as usize);
        }

        None
    }
}

mod test {
    use crate::BlueNoise;

    #[test]
    fn get_points() {
        let mut noise = BlueNoise::new(10, 10, 1.0);
        for x in noise.with_seed(0) {
            println!("{},{}", x.x(), x.y());
        }
    }
}
