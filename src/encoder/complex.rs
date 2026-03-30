use crate::encoder::helpers::BitsRequired;

#[derive(Debug, PartialEq, Eq)]
struct Groups(Vec<Group>);

impl Groups {
    fn new(groups: Vec<Group>) -> Self {
        Self(groups)
    }

    fn from_values(values: &[i32], num_lookahead: usize) -> Self {
        let mut groups = Vec::new();
        let mut start = 0;

        while start < values.len() {
            let mut end = start + 1;

            let v = values[start];
            let (mut min, mut max) = (v, v);
            let mut width = 0;

            while end < values.len() {
                let v = values[end];
                let (new_min, new_max) = (min.min(v), max.max(v));
                let new_width = ((new_max - new_min) as u32).bits_required();

                let len = end - start;
                let cost_extend = group_cost(len + 1, new_width);
                let cost_keep = group_cost(len, width)
                    + new_group_cost_estimated(&values[end..], num_lookahead);
                if cost_keep < cost_extend {
                    break;
                }

                min = new_min;
                max = new_max;
                width = new_width;
                end += 1;
            }

            groups.push(Group::from_values(&values[start..end]));
            start = end;
        }

        Self(groups)
    }
}

fn group_cost(len: usize, width: u8) -> usize {
    len * width as usize
}

fn new_group_cost_estimated(values: &[i32], num_lookahead: usize) -> usize {
    if values.is_empty() {
        return 0;
    }

    let lookahead = values.iter().take(num_lookahead);

    let (mut min, mut max) = (i32::MAX, i32::MIN);
    let mut len = 0;

    for &v in lookahead {
        (min, max) = (min.min(v), max.max(v));
        len += 1;
    }
    let width = ((max - min) as u32).bits_required();
    group_cost(len, width)
}

#[derive(Debug, PartialEq, Eq)]
struct Group {
    pub ref_val: i32,
    pub width: u8,
    pub values: Vec<u32>,
}

impl Group {
    fn from_values(values: &[i32]) -> Self {
        let ref_val = *values.iter().min().unwrap();
        let mut max_diff = u32::MIN;
        let diffs = values
            .iter()
            .map(|v| {
                let diff = (v - ref_val) as u32;
                max_diff = max_diff.max(diff);
                diff
            })
            .collect();
        let width = max_diff.bits_required();

        Group {
            ref_val,
            width,
            values: diffs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping() {
        let mut values = (0_i32..24).collect::<Vec<_>>();
        values[10] = 64;
        values[21] = 128;
        values[22] = 256;
        let actual = Groups::from_values(&values, 4);
        let expected = Groups::new(vec![
            Group {
                ref_val: 0,
                width: 4,
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
            Group {
                ref_val: 64,
                width: 0,
                values: vec![0],
            },
            Group {
                ref_val: 11,
                width: 4,
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
            Group {
                ref_val: 128,
                width: 8,
                values: vec![0, 128],
            },
            Group {
                ref_val: 23,
                width: 0,
                values: vec![0],
            },
        ]);
        assert_eq!(actual, expected);
    }
}
