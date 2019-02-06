use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::StringVecOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use record::RecordTrait;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SortOptions;
use super::SortOptionsValidated;
use validates::ValidationError;

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

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["x"], |p, a| p.xk.push_split(a), "keys to use as columns");
        opt.match_single(&["y"], |p, a| p.yk.push_split(a), "keys to use as rows");
        opt.match_n(&["p"], 2, |p, a| {
            if let Some(_) = p.pins.0.insert(a[0].clone(), a[1].clone()) {
                return ValidationError::message(format!("Pin {} specified twice", a[0]));
            }
            return Result::Ok(());
        }, "keys (and values) to filter to");
        opt.match_single(&["v"], |p, a| p.vk.push_split(a), "keys to use as values (default: whatever keys didn't get used otherwise)");
        opt.add_sub(|p| &mut p.xs, SortOptions::new_options(&["xs"], "sorts for x 'records'"));
        opt.add_sub(|p| &mut p.ys, SortOptions::new_options(&["ys"], "sorts for y 'records'"));
        opt.add(SortOptions::help_options());
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["to-ptable"];
    }

    fn help_msg() -> &'static str {
        return "construct a pivot table from input records";
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (o, Vec::new()),
            |s, e, _w| {
                let (o, cell_tuples) = s;
                let r = e.parse();

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

                return true;
            },
            |s, w| {
                let (o, cell_tuples) = s;

                let (xh, xh_width) = build_header_tree(&o.xk, &o.xs, cell_tuples.iter().map(|(xs, _ys, _v)| xs));
                let (yh, yh_width) = build_header_tree(&o.yk, &o.ys, cell_tuples.iter().map(|(_xs, ys, _v)| ys));

                let width = o.yk.len() + 1 + xh_width;
                let height = o.xk.len() + 1 + yh_width;

                let mut cells: Vec<Vec<_>> = (0..height).map(|_| (0..width).map(|_| ("".to_string(), ' ')).collect()).collect();

                for (i, k) in o.xk.iter().enumerate() {
                    cells[i][o.yk.len()] = (k.to_string(), ' ');
                }
                for (i, k) in o.yk.iter().enumerate() {
                    cells[o.xk.len()][i] = (k.to_string(), ' ');
                }

                xh.visit(0, &mut |width, depth, v| cells[depth][o.yk.len() + 1 + width] = (v.pretty_string(), ' '));
                yh.visit(0, &mut |width, depth, v| cells[o.xk.len() + 1 + width][depth] = (v.pretty_string(), ' '));

                for (xs, ys, v) in cell_tuples.iter() {
                    let x = o.yk.len() + 1 + xh.tag(&xs);
                    let y = o.xk.len() + 1 + yh.tag(&ys);
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
        );
    }
}

#[derive(Default)]
struct HeaderTree<T> {
    arr: Vec<(Record, HeaderTree<T>)>,
    idxs: HashMap<Record, usize>,
    tag: T,
}

impl<T: Default> HeaderTree<T> {
    fn touch(&mut self, zs: &[Record]) {
        zs.iter().fold(self, |pht, v| {
            if let Some(idx) = pht.idxs.get(v) {
                return &mut pht.arr[*idx].1;
            }
            let idx = pht.arr.len();
            pht.arr.push((v.clone(), HeaderTree::default()));
            pht.idxs.insert(v.clone(), idx);
            return &mut pht.arr[idx].1;
        });
    }
}

impl<T: Clone> HeaderTree<T> {
    fn visit<F: FnMut(T, usize, &Record)>(&self, depth: usize, f: &mut F) {
        for (v, ht) in self.arr.iter() {
            f(ht.tag.clone(), depth, v);
            ht.visit(depth + 1, f);
        }
    }

    fn tag(&self, zs: &Vec<Record>) -> T {
        return zs.iter().fold(self, |ht, v| {
            return &ht.arr[ht.idxs[v]].1;
        }).tag.clone();
    }
}

impl<T> HeaderTree<T> {
    fn retag_width<S: Clone, F: FnMut(T, usize, usize) -> S>(self, width: &mut usize, f: &mut F) -> HeaderTree<S> {
        let width0 = *width;
        let arr: Vec<_> = self.arr.into_iter().map(|(v, pht)| {
            return (v, pht.retag_width(width, f));
        }).collect();
        if arr.is_empty() {
            *width += 1;
        }
        let width1 = *width;
        return HeaderTree {
            arr: arr,
            idxs: self.idxs,
            tag: f(self.tag, width0, width1),
        };
    }
}

fn build_header_tree<'a>(zk: &Vec<String>, zsort: &SortOptionsValidated, zss: impl Iterator<Item = &'a Vec<Record>>) -> (HeaderTree<usize>, usize) {
    let mut bucket = zsort.new_bucket();
    let mut already = HashSet::new();
    for zs in zss {
        if already.contains(zs) {
            continue;
        }
        already.insert(zs);

        let mut zr = Record::empty_hash();
        for (k, v) in zk.iter().zip(zs.iter()) {
            zr.set_path(k, v.clone());
        }
        bucket.add(zr, zs);
    }

    let mut pht = HeaderTree::<()>::default();
    while let Some((_, zs)) = bucket.remove_first() {
        pht.touch(zs);
    }

    let mut width = 0;
    let ht = pht.retag_width(&mut width, &mut |(), width0, _width1| width0);
    return (ht, width);
}
