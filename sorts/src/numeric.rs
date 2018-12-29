use SortSimpleBe;
use SortSimpleBeImpl;
use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;

pub type Impl = SortSimpleBeImpl<SimpleImpl>;

pub struct SimpleImpl();

impl SortSimpleBe for SimpleImpl {
    type T = F64SortDishonorProxy;

    fn names() -> Vec<&'static str> {
        return vec!["n", "num", "numeric"];
    }

    fn get(r: Record) -> F64SortDishonorProxy {
        return F64SortDishonorProxy(r.coerce_f64());
    }
}
