// Copyright 2020 Developers of the 'bluenoise-rs' Project
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! bluenoise-rs
//!
//! bluenoise provides an implementation of poisson disk sampling
//! in two dimensions, with `glam` as the underlying maths library.
//! It aims to be fast, well documented and easy to use, taking
//! advantage of [a few optimisations](http://extremelearning.com.au/an-improved-version-of-bridsons-algorithm-n-for-poisson-disc-sampling/)
//! to dramatically speed up compute speed.
//!
//! # Examples
//! ```
//! use bluenoise::BlueNoise;
//! use rand_pcg::Pcg64Mcg;
//!
//! let mut noise = BlueNoise::<Pcg64Mcg>::new(50.0, 50.0, 10.0);
//! let noise = noise.with_samples(10).with_seed(10);
//!
//! for point in noise.take(10) {
//!     println!("{}, {}", point.x, point.y);
//! }
//! ```
//!
//! ```
//! use bluenoise::WrappingBlueNoise;
//! use rand::SeedableRng;
//! use rand_pcg::Pcg64Mcg;
//!
//! let mut noise = WrappingBlueNoise::from_rng(50.0, 50.0, 10.0, Pcg64Mcg::seed_from_u64(10));
//! let noise = noise.with_samples(10);
//!
//! for point in noise.take(10) {
//!     println!("{}, {}", point.x, point.y);
//! }
//! ```

#![deny(
    dead_code,
    missing_docs,
    missing_doc_code_examples,
    unsafe_code,
    unreachable_code,
    trivial_numeric_casts
)]

use std::f32::consts::{FRAC_1_SQRT_2, PI};

use glam::Vec2;
use itertools::Itertools;
use rand::Rng;
use rand::SeedableRng;

/// Provides a source of `BlueNoise` in a given area at some density.
#[derive(Debug, Clone)]
pub struct BlueNoise<R: Rng> {
    width: f32,
    height: f32,
    max_samples: u32,

    /// The minimum radius between points.
    radius: f32,
    radius_squared: f32,

    cell_size: f32,
    grid: Vec<Option<Vec2>>,
    grid_width: usize,
    grid_height: usize,

    /// A list of points that we can generate new
    /// points around.
    active_points: Vec<Vec2>,

    rng: R,
    init: bool,
}

impl<R: Rng + SeedableRng> BlueNoise<R> {
    /// Creates a new instance of `BlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn new(width: f32, height: f32, min_radius: f32) -> Self {
        Self::from_rng(width, height, min_radius, SeedableRng::from_entropy())
    }

    /// Creates a new instance of `BlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    /// * `seed`: Value to seed the rng with
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn from_seed(width: f32, height: f32, min_radius: f32, seed: u64) -> Self {
        Self::from_rng(width, height, min_radius, SeedableRng::seed_from_u64(seed))
    }

    /// A builder function to seed the rng with a specific
    /// value.
    ///
    /// For an example, see the `BlueNoise` examples.
    pub fn with_seed(&mut self, seed: u64) -> &mut Self {
        self.rng = SeedableRng::seed_from_u64(seed);
        self
    }
}

impl<R: Rng> BlueNoise<R> {
    /// Creates a new instance of `BlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    /// * `rng`: Rng to use
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn from_rng(width: f32, height: f32, min_radius: f32, rng: R) -> Self {
        let cell_size = min_radius * FRAC_1_SQRT_2;
        let grid_width = (width / cell_size).ceil() as usize;
        let grid_height = (height / cell_size).ceil() as usize;
        let grid = vec![None; grid_width * grid_height];
        let radius_squared = min_radius * min_radius;

        Self {
            width,
            height,
            max_samples: 4,
            radius: min_radius,
            radius_squared,
            cell_size,
            grid,
            grid_width,
            grid_height,
            active_points: Vec::<Vec2>::default(),
            rng,
            init: false,
        }
    }

    /// A builder function to set the maximum number of
    /// samples to be when attempting to find new points.
    ///
    /// For an example, see the `BlueNoise` examples.
    pub fn with_samples(&mut self, max_samples: u32) -> &mut Self {
        self.max_samples = max_samples;
        self
    }

    /// A builder function to set the minimum radius between
    /// points.
    ///
    /// For an example, see the `BlueNoise` examples.
    pub fn with_min_radius(&mut self, min_radius: f32) -> &mut Self {
        self.radius = min_radius;
        self
    }

    /// Resets the generator to begin creating noise from the beginning.
    /// This will not reset the prng so if you want deterministic ordering,
    /// make sure to set it explicitly.
    ///
    /// ```
    /// use bluenoise::BlueNoise;
    /// use rand_pcg::Pcg64Mcg;
    ///
    /// let mut noise = BlueNoise::<Pcg64Mcg>::new(10.0, 10.0, 1.0);
    /// let first_10 = noise.with_seed(25).take(10).collect::<Vec<_>>();
    ///
    /// // make sure to re-initialise your seed!
    /// noise.reset().with_seed(25);
    /// let reset_10 = noise.take(10).collect::<Vec<_>>();
    ///
    /// assert_eq!(first_10, reset_10);
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.init = false;
        self.active_points.clear();
        for item in &mut self.grid {
            *item = None;
        }
        self
    }

    /// Compute the distance between two points
    fn distance(&self, point: Vec2, target: Vec2) -> f32 {
        point.distance(target)
    }

    /// Check if a position is far enough away from
    /// nearby previously created points.
    fn is_valid(&self, point: Vec2) -> bool {
        // remove anything outside our box
        if point.x < 0.0 || point.x > self.width || point.y < 0.0 || point.y > self.height {
            return false;
        };

        let x_range = {
            let x = (point.x / self.cell_size) as usize;
            x.saturating_sub(2)..(x + 3).min(self.grid_width)
        };
        let y_range = {
            let y = (point.y / self.cell_size) as usize;
            y.saturating_sub(2)..(y + 3).min(self.grid_height)
        };

        x_range.cartesian_product(y_range).all(|(x, y)| {
            // if there is a point, check if it is further than our min radius
            match self
                .grid
                .get(y * self.grid_width + x)
                .expect("Ended up out of bounds when fetching point.")
            {
                Some(target) => self.distance(point, *target) >= self.radius_squared,
                None => true,
            }
        })
    }

    /// Get some nearby point
    fn get_nearby(&mut self, position: Vec2, seed: f32, sample: u32) -> Vec2 {
        let offset = seed + sample as f32 / self.max_samples as f32;
        let theta = 2.0 * PI * offset;
        let radius = self.radius + 0.001;
        Vec2::new(
            position.x + radius * theta.cos(),
            position.y + radius * theta.sin(),
        )
    }

    /// Get the index for a given position
    fn grid_index(&self, position: Vec2) -> usize {
        let y = self.grid_width * (position.y / self.cell_size) as usize;
        let x = (position.x / self.cell_size) as usize;
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
}

impl<R: Rng> Iterator for BlueNoise<R> {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            self.init = true;
            let x = self.rng.gen_range(0.0..self.width);
            let y = self.rng.gen_range(0.0..self.height);
            return Some(self.insert_point(Vec2::new(x, y)));
        }

        while !self.active_points.is_empty() {
            let index = self.rng.gen::<f32>() * (self.active_points.len() - 1) as f32;
            let parent = self.active_points[index as usize];

            let seed = self.rng.gen::<f32>();
            for sample in 0..self.max_samples {
                let point = self.get_nearby(parent, seed, sample);
                if self.is_valid(point) {
                    return Some(self.insert_point(point));
                }
            }

            self.active_points.remove(index as usize);
        }

        None
    }
}

/// Provides a source of `WrappingBlueNoise` in a given area at some
/// density, where the distance between two points wraps around the
/// edges of the box. This can be used to generate tiling blue noise.
#[derive(Debug, Clone)]
pub struct WrappingBlueNoise<R: Rng>(BlueNoise<R>);

impl<R: Rng + SeedableRng> WrappingBlueNoise<R> {
    /// Creates a new instance of `WrappingBlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn new(width: f32, height: f32, min_radius: f32) -> Self {
        Self(BlueNoise::new(width, height, min_radius))
    }

    /// Creates a new instance of `WrappingBlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    /// * `seed`: Value to seed the rng with
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn from_seed(width: f32, height: f32, min_radius: f32, seed: u64) -> Self {
        Self(BlueNoise::from_seed(width, height, min_radius, seed))
    }

    /// A builder function to seed the rng with a specific
    /// value.
    ///
    /// For an example, see the `WrappingBlueNoise` examples.
    pub fn with_seed(&mut self, seed: u64) -> &mut Self {
        self.0.with_seed(seed);
        self
    }
}

impl<R: Rng> WrappingBlueNoise<R> {
    /// Creates a new instance of `WrappingBlueNoise`.
    ///
    /// * `width`: The width of the box to generate inside.
    /// * `height`: The height of the box to generate inside.
    /// * `min_radius`: The minimum distance between points.
    /// * `rng`: Rng to use
    #[must_use = "This is quite expensive to initialise. You can iterate over it to consume it."]
    pub fn from_rng(width: f32, height: f32, min_radius: f32, rng: R) -> Self {
        Self(BlueNoise::from_rng(width, height, min_radius, rng))
    }

    /// A builder function to set the maximum number of
    /// samples to be when attempting to find new points.
    ///
    /// For an example, see the `WrappingBlueNoise` examples.
    pub fn with_samples(&mut self, max_samples: u32) -> &mut Self {
        self.0.with_samples(max_samples);
        self
    }

    /// A builder function to set the minimum radius between
    /// points.
    ///
    /// For an example, see the `WrappingBlueNoise` examples.
    pub fn with_min_radius(&mut self, min_radius: f32) -> &mut Self {
        self.0.with_min_radius(min_radius);
        self
    }

    /// Resets the generator to begin creating noise from the beginning.
    /// This will not reset the prng so if you want deterministic ordering,
    /// make sure to set it explicitly.
    ///
    /// ```
    /// use bluenoise::WrappingBlueNoise;
    /// use rand_pcg::Pcg64Mcg;
    ///
    /// let mut noise = WrappingBlueNoise::<Pcg64Mcg>::new(10.0, 10.0, 1.0);
    /// let first_10 = noise.with_seed(25).take(10).collect::<Vec<_>>();
    ///
    /// // make sure to re-initialise your seed!
    /// noise.reset().with_seed(25);
    /// let reset_10 = noise.take(10).collect::<Vec<_>>();
    ///
    /// assert_eq!(first_10, reset_10);
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.0.reset();
        self
    }

    /// Compute the distance between two points
    fn distance(&self, point: Vec2, target: Vec2) -> f32 {
        let diff = {
            let tmp = (target - point).abs();
            tmp.min(Vec2::new(self.0.width, self.0.height) - tmp)
        };
        diff.length_squared()
    }

    /// Check if a position is far enough away from
    /// nearby previously created points.
    fn is_valid(&self, point: Vec2) -> bool {
        let x_range = {
            let x = (point.x / self.0.cell_size) as isize;
            ((x - 2)..(x + 3)).map(|x| x.rem_euclid(self.0.grid_width as isize) as usize)
        };
        let y_range = {
            let y = (point.y / self.0.cell_size) as isize;
            ((y - 2)..(y + 3)).map(|y| y.rem_euclid(self.0.grid_height as isize) as usize)
        };

        x_range.cartesian_product(y_range).all(|(x, y)| {
            // if there is a point, check if it is further than our min radius
            match self
                .0
                .grid
                .get(y * self.0.grid_width + x)
                .expect("Ended up out of bounds when fetching point.")
            {
                Some(target) => self.distance(point, *target) >= self.0.radius_squared,
                None => true,
            }
        })
    }

    /// Get some nearby point
    fn get_nearby(&mut self, position: Vec2, seed: f32, sample: u32) -> Vec2 {
        let nearby = self.0.get_nearby(position, seed, sample);
        Vec2::new(
            nearby.x.rem_euclid(self.0.width),
            nearby.y.rem_euclid(self.0.height),
        )
    }
}

impl<R: Rng> Iterator for WrappingBlueNoise<R> {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.init {
            self.0.init = true;
            let x = self.0.rng.gen_range(0.0..self.0.width);
            let y = self.0.rng.gen_range(0.0..self.0.height);
            return Some(self.0.insert_point(Vec2::new(x, y)));
        }

        while !self.0.active_points.is_empty() {
            let index = self.0.rng.gen::<f32>() * (self.0.active_points.len() - 1) as f32;
            let parent = self.0.active_points[index as usize];

            let seed = self.0.rng.gen::<f32>();
            for sample in 0..self.0.max_samples {
                let point = self.get_nearby(parent, seed, sample);
                if self.is_valid(point) {
                    return Some(self.0.insert_point(point));
                }
            }

            self.0.active_points.remove(index as usize);
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::{BlueNoise, WrappingBlueNoise};
    use rand_pcg::Pcg64Mcg;

    #[test]
    fn get_points() {
        let noise = BlueNoise::<Pcg64Mcg>::new(100.0, 100.0, 1.0);
        assert!(noise.count() > 0);
    }

    #[test]
    fn get_points_wrapping() {
        let noise = WrappingBlueNoise::<Pcg64Mcg>::new(100.0, 100.0, 1.0);
        assert!(noise.count() > 0);
    }
}
