extern crate alloc;

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use rand::RngExt;
    use ris::*;

    #[test]
    fn test_lis_basic() {
        let x = [2, 1, 4, 3, 5];
        let indices = x.lis_indices();

        assert_eq!(indices, &[1, 3, 4]);
        assert_eq!(x.lis_values(), &[1, 3, 5]);
        assert_eq!(x.lis_refs(), &[&1, &3, &5]);
        assert_eq!(x.lis_length(), 3);
    }

    #[test]
    fn test_lis_empty_and_single() {
        let empty: [i32; 0] = [];
        assert!(empty.lis_indices().is_empty());

        let single = [10];
        assert_eq!(single.lis_indices(), &[0]);
    }

    #[test]
    fn test_lis_sorted() {
        let sorted = [1, 2, 3, 4, 5];
        assert_eq!(sorted.lis_indices(), vec![0, 1, 2, 3, 4]);

        let rev = [5, 4, 3, 2, 1];
        assert_eq!(rev.lis_indices().len(), 1);
        assert_eq!(rev.lis_length(), 1);
    }

    #[test]
    fn test_lis_duplicates() {
        let dups = [2, 2, 2, 2];
        assert_eq!(dups.lis_indices().len(), 1);
        assert_eq!(dups.lis_length(), 1);
    }

    #[test]
    fn test_lis_iterator() {
        let it = (0..10).rev().chain(20..25);
        let res = it.lis();

        assert_eq!(res.len(), 6);

        for i in 0..res.len() - 1 {
            assert!(res[i] < res[i + 1]);
        }

        assert_eq!(res.last(), Some(&24));
    }

    #[test]
    #[cfg(feature = "diff")]
    fn test_diff_basic() {
        let old_list = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let new_list = vec!["a", "c", "b", "e"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        struct Log {
            ops: Vec<String>,
        }
        impl DiffCallback<String, String> for Log {
            fn inserted(&mut self, n: &String) {
                self.ops.push(format!("ins:{}", n));
            }
            fn unchanged(&mut self, _o: &String, _n: &String) {
                self.ops.push("un".to_string());
            }
            fn removed(&mut self, o: &String) {
                self.ops.push(format!("rem:{}", o));
            }
            fn moved(&mut self, _o: &String, n: &String) {
                self.ops.push(format!("mov:{}", n));
            }
        }

        let mut log = Log { ops: Vec::new() };
        diff_by_key(&old_list, |k| k.clone(), &new_list, |k| k.clone(), &mut log);

        // a: unchanged (prefix)
        // d: removed
        // b: moved
        // c: unchanged
        // e: inserted
        assert!(log.ops.contains(&"un".to_string()));
        assert!(log.ops.contains(&"rem:d".to_string()));
        assert!(log.ops.contains(&"ins:e".to_string()));
    }

    #[test]
    #[cfg(feature = "diff")]
    fn test_diff_prefix_suffix_optimization() {
        let old_list = vec!["PRE", "a", "b", "SUF"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let new_list = vec!["PRE", "b", "a", "SUF"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let mut unchanged_count = 0;
        struct Counter<'a>(&'a mut usize);
        impl DiffCallback<String, String> for Counter<'_> {
            fn inserted(&mut self, _: &String) {}
            fn unchanged(&mut self, _: &String, _: &String) {
                *self.0 += 1;
            }
            fn removed(&mut self, _: &String) {}
            fn moved(&mut self, _: &String, _: &String) {}
        }

        diff_by_key(
            &old_list,
            |k| k.clone(),
            &new_list,
            |k| k.clone(),
            &mut Counter(&mut unchanged_count),
        );

        assert_eq!(unchanged_count, 3);
    }

    #[test]
    fn test_lis_random_property() {
        let mut rng = rand::rng();

        fn naive_lis_len(data: &[i32]) -> usize {
            if data.is_empty() {
                return 0;
            }
            let mut dp = vec![1; data.len()];
            for i in 1..data.len() {
                for j in 0..i {
                    if data[j] < data[i] {
                        dp[i] = dp[i].max(dp[j] + 1);
                    }
                }
            }
            *dp.iter().max().unwrap_or(&0)
        }

        for _ in 0..100 {
            let size = 1000;
            let data: Vec<i32> = (0..size).map(|_| rng.random_range(0..1000)).collect();

            let indices = data.lis_indices();

            for i in 0..indices.len().saturating_sub(1) {
                assert!(indices[i] < indices[i + 1]);
            }

            for i in 0..indices.len().saturating_sub(1) {
                assert!(data[indices[i]] < data[indices[i + 1]]);
            }

            let expected_len = naive_lis_len(&data);
            assert_eq!(
                indices.len(),
                expected_len,
                "LIS length mismatch for data: {:?}",
                data
            );

            assert_eq!(data.lis_length(), expected_len);
        }
    }
}
