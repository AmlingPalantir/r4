use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use record::Record;
use record::RecordTrait;

#[derive(Default)]
#[derive(Validates)]
pub struct TwoRecordUnionOption {
    left_prefix: OptionalStringOption,
    right_prefix: OptionalStringOption,
}

impl TwoRecordUnionOption {
    pub fn options<'a>(opt: &mut OptParserView<'a, TwoRecordUnionOption>) {
        opt.sub(|p| &mut p.left_prefix).match_single(&["lp", "left-prefix"], OptionalStringOption::set_str);
        opt.sub(|p| &mut p.right_prefix).match_single(&["rp", "right-prefix"], OptionalStringOption::set_str);
    }
}

impl TwoRecordUnionOptionValidated {
    pub fn union_maybe(&self, r1: Option<Record>, r2: Option<Record>) -> Record {
        fn _union_aux(r: &mut Record, prefix: &Option<String>, r1: Record) {
            match prefix {
                Some(prefix) => {
                    r.set_path(&prefix, r1);
                }
                None => {
                    for (k, v) in r1.expect_hash().into_iter() {
                        r.set_path(&k, v.clone());
                    }
                }
            }
        }

        let mut r = Record::empty_hash();
        if let Some(r1) = r1 {
            _union_aux(&mut r, &self.left_prefix, r1);
        }
        if let Some(r2) = r2 {
            _union_aux(&mut r, &self.right_prefix, r2);
        }
        return r;
    }

    pub fn union(&self, r1: Record, r2: Record) -> Record {
        return self.union_maybe(Some(r1), Some(r2));
    }
}
