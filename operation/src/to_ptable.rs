use opts::parser::OptParserView;
use opts::vals::StringVecOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use record::RecordTrait;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::sort::SortOptions;
use super::sort::SortOptionsValidated;
use validates::Validates;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    xk: StringVecOption,
    yk: StringVecOption,
    pins: UnvalidatedOption<HashMap<String, String>>,
    vk: StringVecOption,
    xs: SortOptions,
    ys: SortOptions,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["to-ptable"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.xk).match_single(&["x"], StringVecOption::push_split);
        opt.sub(|p| &mut p.yk).match_single(&["y"], StringVecOption::push_split);
        opt.match_n(&["p"], 2, |p, a| {
            assert!(p.pins.0.insert(a[0].clone(), a[1].clone()).is_none());
        });
        opt.sub(|p| &mut p.vk).match_single(&["v"], StringVecOption::push_split);
        SortOptions::options(&mut opt.sub(|p| &mut p.xs), &["xs"]);
        SortOptions::options(&mut opt.sub(|p| &mut p.ys), &["ys"]);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                (o, Vec::new()),
                |s, e, _w| {
                    let (o, cell_tuples) = s;

                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            for (k, ve) in o.pins.iter() {
                                let vo = r.get_path(k).coerce_string();
                                let vo = &vo as &str;
                                if ve != vo {
                                    return true;
                                }
                            }

                            let mut rvk = o.vk.clone();
                            if rvk.is_empty() {
                                let mut unused = BTreeSet::new();
                                for k in r.expect_hash().keys() {
                                    unused.insert(k.to_string());
                                }
                                for k in o.xk.iter() {
                                    unused.remove(k);
                                }
                                for k in o.yk.iter() {
                                    unused.remove(k);
                                }
                                for k in o.pins.keys() {
                                    unused.remove(k);
                                }
                                rvk = unused.into_iter().collect();
                            }

                            for vk in rvk {
                                let mut xs = Vec::new();
                                let mut ys = Vec::new();
                                for (zk, zs) in vec![(&o.xk, &mut xs), (&o.yk, &mut ys)] {
                                    for k in zk.iter() {
                                        let v;
                                        if k == "VALUE" {
                                            v = Record::from(&vk as &str);
                                        }
                                        else {
                                            v = r.get_path(&k);
                                        }
                                        zs.push(v);
                                    }
                                }

                                let v = r.get_path(&vk);

                                cell_tuples.push((xs, ys, v));
                            }
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in ToPivotTableStream");
                        }
                    }
                    return true;
                },
                |s, w| {
                    let (o, cell_tuples) = s;

                    let (xh, xh_width) = HeaderTree::build(&o.xk, &o.xs, cell_tuples.iter().map(|(xs, _ys, _v)| xs));
                    let (yh, yh_width) = HeaderTree::build(&o.yk, &o.ys, cell_tuples.iter().map(|(_xs, ys, _v)| ys));

                    let width = o.yk.len() + 1 + xh_width;
                    let height = o.xk.len() + 1 + yh_width;

                    let mut cells: Vec<Vec<_>> = (0..height).map(|_| (0..width).map(|_| ("".to_string(), ' ')).collect()).collect();

                    for (i, k) in o.xk.iter().enumerate() {
                        cells[i][o.yk.len()] = (k.to_string(), ' ');
                    }
                    for (i, k) in o.yk.iter().enumerate() {
                        cells[o.xk.len()][i] = (k.to_string(), ' ');
                    }

                    xh.visit_cells(0, &mut |width, depth, v| cells[depth][o.yk.len() + 1 + width] = (v.pretty_string(), ' '));
                    yh.visit_cells(0, &mut |width, depth, v| cells[o.xk.len() + 1 + width][depth] = (v.pretty_string(), ' '));

                    for (xs, ys, v) in cell_tuples.iter() {
                        let x = o.yk.len() + 1 + xh.width(&xs);
                        let y = o.xk.len() + 1 + yh.width(&ys);
                        cells[y][x] = (v.pretty_string(), ' ');
                    }

                    let mut cells2: Vec<Vec<_>> = (0..=(2 * height)).map(|_| (0..=(2 * width)).map(|_| ("".to_string(), ' ')).collect()).collect();

                    for x in 0..=width {
                        for y in 0..=height {
                            cells2[2 * y][2 * x] = ("+".to_string(), ' ');
                            if x < width {
                                cells2[2 * y][2 * x + 1] = ("".to_string(), '-');
                            }
                            if y < height {
                                cells2[2 * y + 1][2 * x] = ("|".to_string(), ' ');
                            }
                            if x < width && y < height {
                                cells2[2 * y + 1][2 * x + 1] = cells[y][x].clone();
                            }
                        }
                    }

                    super::to_table::dump_table(&cells2, w);
                },
            ),
        );
    }
}

#[derive(Default)]
struct PreHeaderTree {
    arr: Vec<(Record, PreHeaderTree)>,
    idxs: HashMap<Record, usize>,
}

impl PreHeaderTree {
    fn rebuild(self, width: &mut usize) -> HeaderTree {
        let width0 = *width;
        let arr: Vec<_> = self.arr.into_iter().map(|(v, pht)| {
            return (v, pht.rebuild(width));
        }).collect();
        if arr.is_empty() {
            *width += 1;
        }
        return HeaderTree {
            arr: arr,
            idxs: self.idxs,
            width0: width0,
        };
    }
}

struct HeaderTree {
    arr: Vec<(Record, HeaderTree)>,
    idxs: HashMap<Record, usize>,
    width0: usize,
}

impl HeaderTree {
    fn build<'a>(zk: &Vec<String>, zsort: &SortOptionsValidated, zss: impl Iterator<Item = &'a Vec<Record>>) -> (HeaderTree, usize) {
        let mut pairs = Vec::new();
        let mut already = HashSet::new();
        for zs in zss {
            if already.contains(zs) {
                continue;
            }
            already.insert(zs.clone());
            let mut zr = Record::empty_hash();
            for (k, v) in zk.iter().zip(zs.iter()) {
                zr.set_path(k, v.clone());
            }
            pairs.push((zr, zs));
        }

        zsort.sort_aux(&mut pairs);

        let mut pht = PreHeaderTree::default();
        for (_, zs) in pairs {
            zs.iter().fold(&mut pht, |pht, v| {
                if let Some(idx) = pht.idxs.get(v) {
                    return &mut pht.arr[*idx].1;
                }
                let idx = pht.arr.len();
                pht.arr.push((v.clone(), PreHeaderTree::default()));
                pht.idxs.insert(v.clone(), idx);
                return &mut pht.arr[idx].1;
            });
        }

        let mut width = 0;
        let ht = pht.rebuild(&mut width);
        return (ht, width);
    }

    fn visit_cells<F: FnMut(usize, usize, &Record)>(&self, depth: usize, f: &mut F) {
        for (v, ht) in self.arr.iter() {
            f(ht.width0, depth, v);
            ht.visit_cells(depth + 1, f);
        }
    }

    fn width(&self, zs: &Vec<Record>) -> usize {
        return zs.iter().fold(self, |ht, v| {
            return &ht.arr[ht.idxs[v]].1;
        }).width0;
    }
}
