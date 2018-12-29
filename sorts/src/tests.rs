use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaChaRng;

#[test]
fn test_reorder_simple() {
    let mut a = vec![0, 1, 2, 3, 4, 5];
    let idxs = vec![4, 1, 5, 0, 3, 2];

    super::reorder(&mut a, &idxs);

    assert_eq!(a, idxs);
}

#[test]
fn test_reorder_stress() {
    let mut rng = ChaChaRng::from_seed([0; 32]);

    for sz in &[0, 1, 2, 3, 8, 10, 100, 1000] {
        let sz = *sz;
        for _rep in 0..100 {
            let mut a: Vec<usize> = (0..sz).collect();
            let mut idxs = a.clone();

            idxs.shuffle(&mut rng);

            super::reorder(&mut a, &idxs);

            assert_eq!(a, idxs);
        }
    }
}
