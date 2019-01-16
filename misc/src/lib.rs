use std::rc::Rc;

#[derive(Clone)]
#[derive(Debug)]
pub enum Either<L, R> {
    Right(R),
    Left(L),
}

impl<L, R> Either<L, R> {
    pub fn convert_r_mut<F: FnOnce(&L) -> R>(&mut self, f: F) -> &mut R {
        // Arggh, I wish f could take L...
        let new_r;
        match self {
            Either::Right(r) => {
                return r;
            }
            Either::Left(l) => {
                new_r = f(l);
            }
        }
        *self = Either::Right(new_r);
        match self {
            Either::Right(r) => {
                return r;
            }
            Either::Left(_l) => {
                unreachable!();
            }
        }
    }

    pub fn map_left<L2, F: FnOnce(L) -> L2>(self, f: F) -> Either<L2, R> {
        return match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        };
    }

    pub fn map_right<R2, F: FnOnce(R) -> R2>(self, f: F) -> Either<L, R2> {
        return match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        };
    }
}

impl<T> Either<T, T> {
    pub fn join(self) -> T {
        return match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        };
    }
}

pub struct PointerRc<T: ?Sized>(pub Rc<T>);

impl<T: ?Sized> PartialEq for PointerRc<T> {
    fn eq(&self, rhs: &PointerRc<T>) -> bool {
        return Rc::ptr_eq(&self.0, &rhs.0);
    }
}

impl<T: ?Sized> Eq for PointerRc<T> {
}
