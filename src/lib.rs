use ordered_float::OrderedFloat;
use std::cmp::Ordering;

// Implementing the Centroid data structure

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct Centroid {
    mean: OrderedFloat<f64>,
    weight: OrderedFloat<f64>,
}

impl PartialOrd for Centroid {
    fn partial_cmp(&self, other: &Centroid) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Centroid {
    fn cmp(&self, other: &Centroid) -> Ordering {
        self.mean.cmp(&other.mean)
    }
}

impl Centroid {
    pub fn new(mean: f64, weight: f64) -> Self {
        Centroid {
            mean: OrderedFloat::from(mean),
            weight: OrderedFloat::from(weight),
        }
    }

    #[inline]
    pub fn mean(&self) -> f64 {
        self.mean.into_inner()
    }

    #[inline]
    pub fn weight(&self) -> f64 {
        self.weight.into_inner()
    }

    pub fn update(&mut self, value: f64, weight: f64) -> (f64, f64) {
        let _weight: f64 = self.weight();
        let _mean: f64 = self.mean();

        self.weight = OrderedFloat::from(weight + _weight);
        self.mean = OrderedFloat::from((_mean * _weight + value) / self.weight());

        (self.mean(), self.weight())
    }
}

impl Default for Centroid {
    fn default() -> Self {
        Centroid::new(0.0, 1.0)
    }
}

/// T-Digest to be operated on.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct TDigest {
    centroids: Vec<Centroid>,
    max_size: usize,
    sum: OrderedFloat<f64>,
    count: OrderedFloat<f64>,
    max: OrderedFloat<f64>,
    min: OrderedFloat<f64>,
}

impl TDigest {
    pub fn new_with_size(max_size: usize) -> Self {
        TDigest {
            centroids: Vec::new(),
            max_size,
            sum: OrderedFloat::from(0.0),
            count: OrderedFloat::from(0.0),
            max: OrderedFloat::from(std::f64::NAN),
            min: OrderedFloat::from(std::f64::NAN),
        }
    }

    pub fn new(
        centroids: Vec<Centroid>,
        max_size: usize,
        sum: f64,
        count: f64,
        max: f64,
        min: f64,
    ) -> Self {
        if centroids.len() <= max_size {
            TDigest {
                centroids,
                max_size,
                sum: OrderedFloat::from(sum),
                count: OrderedFloat::from(count),
                max: OrderedFloat::from(max),
                min: OrderedFloat::from(min),
            }
        } else {
            let sz = centroids.len();
            let digests: Vec<TDigest> = vec![
                TDigest::new_with_size(100),
                TDigest::new(centroids, sz, sum, count, max, min),
            ];
            Self::merge_digests(digests)
        }
    }

    #[inline]
    pub fn mean(&self) -> f64 {
        let count_: f64 = self.count.into_inner();
        let sum_: f64 = self.sum.into_inner();

        if count_ > 0.0 {
            sum_ / count_
        } else {
            std::f64::NAN
        }
    }

    #[inline]
    pub fn sum(&self) -> f64 {
        self.sum.into_inner()
    }

    #[inline]
    pub fn count(&self) -> f64 {
        self.count.into_inner()
    }

    #[inline]
    pub fn max(&self) -> f64 {
        self.max.into_inner()
    }

    #[inline]
    pub fn min(&self) -> f64 {
        self.min.into_inner()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.centroids.is_empty()
    }

    #[inline]
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl Default for TDigest {
    fn default() -> Self {
        TDigest {
            centroids: Vec::new(),
            max_size: 100,
            sum: OrderedFloat::from(0.0),
            count: OrderedFloat::from(0.0),
            max: OrderedFloat::from(std::f64::NAN),
            min: OrderedFloat::from(std::f64::NAN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_centroid() {
        let c = Centroid::new(5.0, 1.0);
        assert_eq!(c.mean(), 5.0);
        assert_eq!(c.weight(), 1.0);
    }

    #[test]
    fn test_update_centroid() {
        let mut c = Centroid::new(5.0, 1.0);
        let (new_mean, new_weight) = c.update(7.0, 2.0);
        assert_eq!(new_mean, (5.0 * 1.0 + 7.0) / 3.0); // New mean
        assert_eq!(new_weight, 3.0); // New weight
    }

    #[test]
    fn test_partial_cmp() {
        let c1 = Centroid::new(5.0, 1.0);
        let c2 = Centroid::new(7.0, 1.0);
        assert!(c1 < c2);

        let c3 = Centroid::new(3.0, 5.0);
        let c4 = Centroid::new(5.0, 3.0);
        assert!(c4 > c3);
    }

    #[test]
    fn test_ord() {
        let c1 = Centroid::new(5.0, 1.0);
        let c2 = Centroid::new(7.0, 1.0);
        assert_eq!(c1.cmp(&c2), Ordering::Less);
        assert_eq!(c2.cmp(&c1), Ordering::Greater);
        let c3 = Centroid::new(5.0, 2.0);
        assert_eq!(c1.cmp(&c3), Ordering::Equal);
    }

    #[test]
    fn test_clone_copy() {
        let c1 = Centroid::new(5.0, 1.0);
        let c2 = c1; // Copy
        assert_eq!(c1, c2);
        let c3 = c1;
        assert_eq!(c1, c3);
    }

    #[test]
    fn test_debug() {
        let c = Centroid::new(5.0, 1.0);
        let debug_str = format!("{:?}", c);
        let actual_str = "Centroid { mean: OrderedFloat(5.0), weight: OrderedFloat(1.0) }";
        assert_eq!(debug_str, actual_str);
    }

    #[test]
    fn test_eq() {
        let c1 = Centroid::new(5.0, 1.0);
        let c2 = Centroid::new(5.0, 1.0);
        assert_eq!(c1, c2);
        let c3 = Centroid::new(5.0, 2.0);
        assert_ne!(c1, c3); // Centroids are only equal if they have the same mean and weight
    }
}
