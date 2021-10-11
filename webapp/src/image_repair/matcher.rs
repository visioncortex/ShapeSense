use std::{collections::{HashMap, HashSet}};

use permutator::{Combination, factorial, multiply_factorial};
use visioncortex::PointF64;

use crate::{image_repair::SquareDistanceMatrix, util::console_log_util};

use super::{MatchItem, MatchItemSet, Matching};

/// Given a set of an even number of 2-D points and direction vectors at those points,
/// find a complete, disjoint, pair matching of those points such that the sum of distances between the pairs is at minimum.
pub struct Matcher;

// API
impl Matcher {
    /// Find the overall 'optimal' matching. Definition of optimality is to be determined.
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

    /// Find all possible matchings for each possible partition.
    /// The behavior is undefined unless 'match_items' contains n items where n is even and n>0.
    pub fn find_all_possible_matchings(match_items: MatchItemSet) -> Vec<Matching> {
        let len = match_items.len();
        assert_eq!(len % 2, 0);
        assert!(len > 0);

        let indices: Vec<usize> = (0..len).into_iter().collect();

        // nCr
        let (n, r) = (len, len >> 1);
        let num_combinations = factorial(n) / multiply_factorial(r, n-r);
        // Only interested in the first half of the nCr space (second half is equivalent)
        let matchings_with_variances: Vec<(Matching, f64)> = indices.combination(r).take(num_combinations >> 1).map(|set1_indices| {
                let set1_indices: HashSet<usize> = set1_indices.into_iter().copied().collect();
                let (mut set1, mut set2) = (MatchItemSet::new(), MatchItemSet::new());
                for i in 0..len {
                    if set1_indices.contains(&i) {
                        set1.push_as_is(match_items[i]);
                    } else {
                        set2.push_as_is(match_items[i]);
                    }
                }

                let variance = Self::calculate_average_variance(&set1, &set2);

                let distance_matrix = SquareDistanceMatrix::from_two_sets(&set1, &set2);

                let index_matching = distance_matrix.into_matching();

                (
                    Matching::from_pairs(index_matching.into_iter()
                                                   .map(|(index1, index2)| {
                                                       (set1[index1].id, set2[index2].id)
                                                    })
                                                    .collect()),
                    variance
                )
            })
            .collect();

        // Keep unique matchings only
        let mut unique_matchings_with_lowest_variances = HashMap::<Matching, f64>::new();
        matchings_with_variances.into_iter()
                                .for_each(|(matching, variance)| {
                                    let current_variance = unique_matchings_with_lowest_variances.get(&matching).cloned().unwrap_or(f64::NAN);
                                    if current_variance.is_nan() || variance < current_variance {
                                        unique_matchings_with_lowest_variances.insert(matching, variance);
                                    }
                                });

        // Convert to vec
        let mut matchings_with_variances: Vec<(Matching, f64)> = unique_matchings_with_lowest_variances.into_iter().collect();

        // Sort by variance
        matchings_with_variances.sort_by(|(_, variance1), (_, variance2)| variance1.partial_cmp(variance2).unwrap());

        // Keep only matchings
        matchings_with_variances.into_iter().map(|(matching, _)| matching).collect()
    }
}

// Helper functions
impl Matcher {
    fn calculate_average_variance(set1: &MatchItemSet, set2: &MatchItemSet) -> f64 {
        let calculate_average_direction = |set: &MatchItemSet| {
            let len = set.len();
            let sum_direction: PointF64 = set.iter()
                                             .fold(PointF64::default(), |acc, item2| {
                                                acc + item2.direction
                                             });
            let average_direction = sum_direction / (len as f64);
            average_direction.get_normalized()
        };

        let calculate_variance = |set: &MatchItemSet| {
            let len = set.len();
            if len == 1 {
                return 0.0;
            }

            let average_direction = calculate_average_direction(set);
            let sum_distances: f64 = set.iter()
                                   .map(|item| {
                                       item.direction.get_normalized().distance_to(average_direction)
                                   })
                                   .sum();
            sum_distances / (len - 1) as f64
        };

        let (variance1, variance2) = (calculate_variance(set1), calculate_variance(set2));
        (variance1 + variance2) / 2.0
    }

    // ============================================================================================================

    fn partition(items: MatchItemSet, direction_difference_threshold: f64) -> (MatchItemSet, MatchItemSet) {
        let (set1, set2) = Self::partition_by_direction(&items, direction_difference_threshold);

        let (set1, set2) = if !set1.is_empty() && !set2.is_empty() {
            (set1, set2)
        } else {
            Self::partition_by_distance(items)
        };

        Self::force_balance_partition(set1, set2)
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

    fn force_balance_partition(mut set1: MatchItemSet, mut set2: MatchItemSet) -> (MatchItemSet, MatchItemSet) {
        while set1.len() != set2.len() {
            let (set1_len, set2_len) = (set1.len(), set2.len());
            if set1_len > set2_len {
                set2.push_as_is(set1.remove(set1_len-1));
            } else {
                set1.push_as_is(set2.remove(set2_len-1));
            }
        }

        (set1, set2)
    }
}