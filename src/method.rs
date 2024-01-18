use super::broadcast::{broadcasting_binary_op, general_broadcasting};
use super::DTensor;
use crate::TensorType;

pub trait Distance {}

#[macro_export]
macro_rules! dim_max {
    ($d1:expr, $d2:expr) => {
        <$d1 as DimMax<$d2>>::Output
    };
}

fn convert_iopsf<A: Clone, B: Clone>(f: impl Fn(A, B) -> A) -> impl FnMut((&mut A, &B)) {
    move |(x, y)| *x = f(x.clone(), y.clone())
}

fn clone_opsf<A: Clone, B: Clone, C>(f: impl Fn(A, B) -> C) -> impl FnMut((&mut A, &B)) -> C {
    move |(x, y)| f(x.clone(), y.clone())
}

macro_rules! impl_binary_op {
    ($trt:ident, $mth:ident) => {
        impl<A> std::ops::$trt<&DTensor<A>> for DTensor<A>
        where
            A: std::ops::$trt<A, Output = A> + TensorType,
        {
            type Output = DTensor<A>;
            fn $mth(self, rhs: &DTensor<A>) -> Self::Output {
                let lhs = self;
                if lhs.shape() == rhs.shape() {
                    lhs.iter().zip2(rhs.iter()).ops(convert_iopsf(A::$mth));
                    lhs
                } else {
                    let broadcast_shape =
                        broadcasting_binary_op::<A>(lhs.shape(), rhs.shape()).unwrap();

                    let l_broadcast = broadcast_shape == *lhs.shape();
                    let r_broadcast = broadcast_shape == *rhs.shape();

                    let v = match (l_broadcast, r_broadcast) {
                        (true, true) => {
                            lhs.iter().zip2(rhs.iter()).ops(convert_iopsf(A::$mth));
                            lhs
                        }
                        (true, false) => {
                            lhs.iter()
                                .zip2(rhs.broadcast_with(&broadcast_shape).unwrap().iter())
                                .ops(convert_iopsf(A::$mth));
                            lhs
                        }
                        (false, true) => lhs
                            .broadcast_with(&broadcast_shape)
                            .unwrap()
                            .iter()
                            .zip2(rhs.iter())
                            .map(clone_opsf(A::$mth))
                            .collect_tensor(lhs.dim.shape().clone()),
                        (false, false) => lhs
                            .broadcast_with(&broadcast_shape)
                            .unwrap()
                            .iter()
                            .zip2(rhs.broadcast_with(&broadcast_shape).unwrap().iter())
                            .map(clone_opsf(A::$mth))
                            .collect_tensor(lhs.dim.shape().clone()),
                    };
                    v
                }
            }
        }

        impl<A> std::ops::$trt<DTensor<A>> for DTensor<A>
        where
            A: std::ops::$trt<A, Output = A> + TensorType,
        {
            type Output = DTensor<A>;
            fn $mth(self, rhs: DTensor<A>) -> Self::Output {
                let lhs = self;
                if lhs.shape() == rhs.shape() {
                    lhs.iter().zip2(rhs.iter()).ops(convert_iopsf(A::$mth));
                    lhs
                } else {
                    let broadcast_shape =
                        broadcasting_binary_op::<A>(lhs.shape(), rhs.shape()).unwrap();

                    let l_broadcast = broadcast_shape == *lhs.shape();
                    let r_broadcast = broadcast_shape == *rhs.shape();

                    let v = match (l_broadcast, r_broadcast) {
                        (true, true) => {
                            lhs.iter().zip2(rhs.iter()).ops(convert_iopsf(A::$mth));
                            lhs
                        }
                        (true, false) => {
                            lhs.iter()
                                .zip2(rhs.broadcast_with(&broadcast_shape).unwrap().iter())
                                .ops(convert_iopsf(A::$mth));
                            lhs
                        }
                        (false, true) => lhs
                            .broadcast_with(&broadcast_shape)
                            .unwrap()
                            .iter()
                            .zip2(rhs.iter())
                            .map(clone_opsf(A::$mth))
                            .collect_tensor(lhs.dim.shape().clone()),
                        (false, false) => lhs
                            .broadcast_with(&broadcast_shape)
                            .unwrap()
                            .iter()
                            .zip2(rhs.broadcast_with(&broadcast_shape).unwrap().iter())
                            .map(clone_opsf(A::$mth))
                            .collect_tensor(lhs.dim.shape().clone()),
                    };
                    v
                }
            }
        }

        impl<A> std::ops::$trt<&DTensor<A>> for &DTensor<A>
        where
            A: std::ops::$trt<A, Output = A> + TensorType,
        {
            type Output = DTensor<A>;
            fn $mth(self, rhs: &DTensor<A>) -> Self::Output {
                if self.shape() == rhs.shape() {
                    self.iter()
                        .zip2(rhs.iter())
                        .map(clone_opsf(A::$mth))
                        .collect_tensor(rhs.dim.shape().clone())
                } else {
                    let (lhs, rhs2) = general_broadcasting::<A>(&self, &rhs).unwrap();
                    lhs.iter()
                        .zip2(rhs2.iter())
                        .map(clone_opsf(A::$mth))
                        .collect_tensor(lhs.dim.shape().clone())
                }
            }
        }

        impl<A> std::ops::$trt<DTensor<A>> for &DTensor<A>
        where
            A: std::ops::$trt<A, Output = A> + TensorType,
        {
            type Output = DTensor<A>;
            fn $mth(self, rhs: DTensor<A>) -> Self::Output {
                if self.shape() == rhs.shape() {
                    self.iter()
                        .zip2(rhs.iter())
                        .map(clone_opsf(A::$mth))
                        .collect_tensor(rhs.dim.shape().clone())
                } else {
                    let (lhs, rhs2) = general_broadcasting::<A>(&self, &rhs).unwrap();
                    lhs.iter()
                        .zip2(rhs2.iter())
                        .map(clone_opsf(A::$mth))
                        .collect_tensor(lhs.dim.shape().clone())
                }
            }
        }
    };
}

impl<A> PartialEq<DTensor<A>> for DTensor<A>
where
    A: TensorType,
{
    fn eq(&self, other: &DTensor<A>) -> bool {
        if self.shape() != other.shape() {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
            if *a != *b {
                return false;
            }
        }
        return true;
    }
}

impl_binary_op!(Add, add); // +
impl_binary_op!(Sub, sub); // -
impl_binary_op!(Mul, mul); // *
impl_binary_op!(Div, div); // /
impl_binary_op!(Rem, rem); // %
impl_binary_op!(BitAnd, bitand); // &
impl_binary_op!(BitOr, bitor); // |
impl_binary_op!(BitXor, bitxor); // ^
impl_binary_op!(Shl, shl); // <<
impl_binary_op!(Shr, shr); // >>

mod tests {
    use super::super::{arr, mat};
    use super::*;
    #[test]
    fn test_add() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        println!("m1 dim:{:?}", m1.dim);
        println!("m1 {:?}", m1);
        println!("m1 stride:{:?}", m1.dim.stride);
        let m2 = arr(&[1.0, 2.0, 3.0]);

        let m3 = m1 + &m2;
        println!("m3:{:?}", m3);
    }

    #[test]
    fn test_sub() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        println!("m1 dim:{:?}", m1.dim);
        println!("m1 {:?}", m1.as_slice());
        println!("m1 stride:{:?}", m1.dim.stride);
        let m2 = arr(&[1.0, 2.0, 3.0]);

        let m3 = &m1 - &m2;
        println!("m3:{:?}", m3);
    }

    #[test]
    fn test_mul() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        println!("m1 dim:{:?}", m1.dim);
        println!("m1 {:?}", m1);
        println!("m1 stride:{:?}", m1.dim.stride);
        let m2 = arr(&[1.0, 2.0, 3.0]);

        let m3 = m1 * &m2;
        println!("m3:{:?}", m3);
    }

    #[test]
    fn test_div() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        println!("m1 dim:{:?}", m1.dim);
        println!("m1 {:?}", m1);
        println!("m1 stride:{:?}", m1.dim.stride);
        let m2 = arr(&[1.0, 2.0, 3.0]);

        let m3 = m1 / &m2;
        println!("m3:{:?}", m3);
    }

    #[test]
    fn test_rem() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        println!("m1 dim:{:?}", m1.dim);
        println!("m1 {:?}", m1);
        println!("m1 stride:{:?}", m1.dim.stride);
        let m2 = arr(&[1.0, 2.0, 3.0]);

        let m3 = m1 % m2;
        println!("m3:{:?}", m3);
    }

    #[test]
    fn test_eq() {
        let m1 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        let m2 = mat(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
        ]);
        let m3 = arr(&[1.0, 2.0, 3.0]);

        println!("{}", m1 == m2);
        println!("{}", m2 == m3);
    }
}
