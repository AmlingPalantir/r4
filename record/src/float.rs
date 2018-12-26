use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone)]
pub struct F64HashDishonorProxy(pub f64);

impl F64HashDishonorProxy {
    pub(crate) fn to_json_string(&self) -> String {
        return serde_json::to_string(&serde_json::Number::from_f64(self.0)).unwrap();
    }
}

// Ouch, if it comes down to Hash/Eq for F64HashDishonorProxy you've really asked for it,
// but we do the least bad, least insane thing we can...

impl Eq for F64HashDishonorProxy {
}

impl PartialEq for F64HashDishonorProxy {
    fn eq(&self, other: &F64HashDishonorProxy) -> bool {
        return self.to_json_string() == other.to_json_string();
    }
}

impl Hash for F64HashDishonorProxy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_json_string().hash(state);
    }
}

#[derive(Clone)]
pub struct F64SortDishonorProxy(pub f64);

impl PartialEq for F64SortDishonorProxy {
    fn eq(&self, other: &F64SortDishonorProxy) -> bool {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0 == other.0;
    }
}

impl PartialOrd for F64SortDishonorProxy {
    fn partial_cmp(&self, other: &F64SortDishonorProxy) -> Option<Ordering> {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0.partial_cmp(&other.0);
    }
}

impl Eq for F64SortDishonorProxy {
}

impl Ord for F64SortDishonorProxy {
    fn cmp(&self, other: &F64SortDishonorProxy) -> Ordering {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0.partial_cmp(&other.0).unwrap();
    }
}
