use crate::{image_repair::SquareDistanceMatrix, util::console_log_util};

use super::{MatchItem, MatchItemSet, Matching};

/// Given a set of an even number of 2-D points and direction vectors at those points,
/// find a complete, disjoint, pair matching of those points such that the sum of distances between the pairs is at minimum.
pub struct Matcher;

// API
impl Matcher {
    /// Find a complete, disjoint, pair matching of those points such that the sum of distances between the pairs is at minimum.
    /// The behavior is undefined unless 'match_items' contains n items where n is even and n>0.
    /// 'direction_difference_threshold': [0.0, 1.0]
    pub fn find_matching(match_items: MatchItemSet, direction_difference_threshold: f64) -> Matching {
        let len = match_items.len();
        assert_eq!(len % 2, 0);
        assert!(len > 0);

        let (set1, set2) = Self::partition(match_items, direction_difference_threshold);

        let distance_matrix = SquareDistanceMatrix::from_two_sets(&set1, &set2);

        let index_matching = distance_matrix.into_matching();

        Matching::from_pairs(index_matching.into_iter().map(|(index1, index2)| {(set1[index1].id, set2[index2].id)}).collect())
    }
}

// Helper functions
impl Matcher {
    fn partition(items: MatchItemSet, direction_difference_threshold: f64) -> (MatchItemSet, MatchItemSet) {
        let (set1, set2) = Self::partition_by_direction(&items, direction_difference_threshold);
        if !set1.is_empty() && !set2.is_empty() {
            (set1, set2)
        } else {
            Self::partition_by_distance(items)
        }
    }

    /// Partition (deep-copying items) 'items' into 2 sets that are complete and disjoint.
    /// Each item takes turn to be a candidate for the reference item, 'items' is partitioned
    /// into 1 set of items that are similar to the reference and another set that are not.
    /// The order of the 2 sets is arbitrary.
    /// The 2 sets have the exact same number of items.
    fn partition_by_direction(items: &MatchItemSet, direction_difference_threshold: f64) -> (MatchItemSet, MatchItemSet) {
        // Find the most balanced partition
        items.iter()
             .fold((MatchItemSet::new(), MatchItemSet::new()), |(best_set1, best_set2), item| {
                if best_set1.is_empty() || best_set2.is_empty() {
                    return Self::get_one_partition(items, item, direction_difference_threshold);
                }
                let (best_set1_len, best_set2_len) = (best_set1.len(), best_set2.len());
                // The first seen balanced sets will be the ones that are used finally.
                if best_set1_len == best_set2_len {
                    return (best_set1, best_set2);
                }
                let (set1, set2) = Self::get_one_partition(items, item, direction_difference_threshold);
                let (set1_len, set2_len) = (set1.len(), set2.len());

                let best_sets_diff = std::cmp::max(best_set1_len, best_set2_len) - std::cmp::min(best_set1_len, best_set2_len);
                let current_sets_diff = std::cmp::max(set1_len, set2_len) - std::cmp::min(set1_len, set2_len);

                if best_sets_diff <= current_sets_diff {
                    (best_set1, best_set2)
                } else {
                    (set1, set2)
                }
            })
    }

    /// Use 'reference_item' to partition 'items' into 2 sets.
    /// Items whose direction is close (euclidean distance) to the direction of 'reference_item' are put in a set.
    /// Closeness is defined by 'direction_difference_threshold', which is normalized to [0.0, 1.0].
    fn get_one_partition(items: &MatchItemSet, reference_item: &MatchItem, direction_difference_threshold: f64) -> (MatchItemSet, MatchItemSet) {
        let reference_direction = reference_item.direction.get_normalized();
        let (mut set1, mut set2) = (MatchItemSet::new(), MatchItemSet::new());

        for &item in items.iter() {
            let direction_difference = reference_direction.distance_to(item.direction.get_normalized());
            // Originally in [0.0, 2.0] (2.0 at perfectly opposite directions)
            let normalized_direction_difference = direction_difference / 2.0;

            if normalized_direction_difference <= direction_difference_threshold {
                set1.push_as_is(item);
            } else {
                set2.push_as_is(item);
            }
        }

        (set1, set2)
    }

    fn partition_by_distance(mut items: MatchItemSet) -> (MatchItemSet, MatchItemSet) {
        let find_furthest_point_from_index = |items: &MatchItemSet, src_index: usize| {
            let src = items[src_index].point;
            items.iter().enumerate().fold((0, -1.0), |(furthest_point_index, furthest_distance), (current_point_index, item)| {
                let current_distance = src.distance_to(item.point);
                if current_point_index == src_index || furthest_distance >= current_distance {
                    (furthest_point_index, furthest_distance)
                } else {
                    (current_point_index, current_distance)
                }
            }).0
        };
        
        // Keep moving the furthest pair into set1 and set2
        let (mut set1, mut set2) = (MatchItemSet::new(), MatchItemSet::new());
        while !items.is_empty() {
            let mut prev = 0;
            let mut curr = find_furthest_point_from_index(&items, prev);

            let mut i = 10; // Upper limit of iteration
            while i > 0 {
                let next = find_furthest_point_from_index(&items, curr);
                if prev == next {
                    curr = next;
                    break;
                }
                prev = curr;
                curr = next;

                i -= 1;
            }

            set1.push_as_is(items.remove(prev));
            set2.push_as_is(items.remove(curr));
        }

        (set1, set2)
    }
}