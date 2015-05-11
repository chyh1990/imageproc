use nalgebra::*;
use geo::{Point, Pointf};

#[derive(Debug, Clone)]
pub struct Affine2D {
    pub t: Mat3<f32>,
    pub t_inv: Mat3<f32>
}

impl Affine2D {
    fn from_mat(t: Mat3<f32>) -> Option<Affine2D> {
        let ti = t.inv();
        match ti {
            Some(ti) => Some(Affine2D {
                t: t,
                t_inv: ti
            }),
            _ => None
        }
    }

    #[allow(non_snake_case)]
    fn solve_affine(A: DMat<f32>, b: DVec<f32>) -> Option<DVec<f32>> {
        if A.nrows() > A.ncols() {
            let At = A.transpose();
            let b = At.clone() * b;
            let Ainv = (At * A).inv();
            match Ainv {
                Some(m) => Some(m * b),
                _ => None
            }
        } else {
            Some(A.inv().unwrap() * b)
        }
    }

    pub fn affine_from_points(src: &[Pointf], dst: &[Pointf]) -> Option<Affine2D> {
        if src.len() != dst.len() || src.len() < 3 {
            return None;
        }
        let n = src.len();
        let mut m: DMat<f32> = DMat::new_zeros(2 * n, 6);
        let mut b: DVec<f32> = DVec::new_zeros(2 * n);
        for i in 0..n as usize {
            m[(i, 0)] = src[i].x;
            m[(i, 1)] = src[i].y;
            m[(i, 2)] = 1f32;
            m[(i + n, 3 + 0)] = src[i].x;
            m[(i + n, 3 + 1)] = src[i].y;
            m[(i + n, 3 + 2)] = 1f32;
            b[i] = dst[i].x;
            b[n + i] = dst[i].y;
        }
        match Affine2D::solve_affine(m, b) {
            Some(x) => {
                let t: Mat3<f32> = Mat3::new(
                    x[0], x[1], x[2],
                    x[3], x[4], x[5],
                    0f32, 0f32, 1f32
                    );
                Affine2D::from_mat(t)
            }
            _ => None
        }
    }

    pub fn nonreflect_similarity_from_points(src: &[Pointf], dst: &[Pointf]) -> Option<Affine2D> {
        if src.len() != dst.len() || src.len() < 2 {
            return None;
        }
        let n = src.len();
        let mut m: DMat<f32> = DMat::new_zeros(2 * n, 4);
        let mut b: DVec<f32> = DVec::new_zeros(2 * n);
        for i in 0..n as usize {
            b[i] = dst[i].x;
            b[n + i] = dst[i].y;
        }
        for i in 0..n as usize {
            m[(i, 0)] = src[i].x;
            m[(i, 1)] = src[i].y;
            m[(i, 2)] = 1f32;
            m[(i, 3)] = 0f32;

            m[(n + i, 0)] = src[i].y;
            m[(n + i, 1)] = -src[i].x;
            m[(n + i, 2)] = 0f32;
            m[(n + i, 3)] = 1f32;
        }
        match Affine2D::solve_affine(m, b) {
            Some(x) => {
                let t: Mat3<f32> = Mat3::new(
                    x[0], -x[1], 0f32,
                    x[1],  x[0], 0f32,
                    x[2],  x[3], 1f32
                    );
                let t: Mat3<f32> = Mat3::new(
                    x[0], x[1], x[2],
                    -x[1],x[0], x[3],
                    0f32, 0f32, 1f32
                    );
                Affine2D::from_mat(t)
            }
            _ => None
        }
    }

    pub fn map_point(&self, src: Pointf) -> Pointf {
        Point {
            x: self.t[(0,0)] * src.x + self.t[(0,1)] * src.y + self.t[(0,2)],
            y: self.t[(1,0)] * src.x + self.t[(1,1)] * src.y + self.t[(1,2)],
        }
    }

    pub fn map_point_inv(&self, src: Pointf) -> Pointf {
        Point {
            x: self.t_inv[(0,0)] * src.x + self.t_inv[(0,1)] * src.y + self.t_inv[(0,2)],
            y: self.t_inv[(1,0)] * src.x + self.t_inv[(1,1)] * src.y + self.t_inv[(1,2)],
        }
    }

    pub fn apply(&self, raw: [f32; 3]) -> [f32; 3] {
        let mut m = [0f32; 3];
        m[0] = self.t.m11 * raw[0] + self.t.m12 * raw[1] + self.t.m13 * raw[2];
        m[1] = self.t.m21 * raw[0] + self.t.m22 * raw[1] + self.t.m23 * raw[2];
        m[2] = self.t.m31 * raw[0] + self.t.m32 * raw[1] + self.t.m33 * raw[2];
        m
    }

    pub fn apply_inv(&self, raw: [f32; 3]) -> [f32; 3] {
        let mut m = [0f32; 3];
        m[0] = self.t_inv.m11 * raw[0] + self.t_inv.m12 * raw[1] + self.t_inv.m13 * raw[2];
        m[1] = self.t_inv.m21 * raw[0] + self.t_inv.m22 * raw[1] + self.t_inv.m23 * raw[2];
        m[2] = self.t_inv.m31 * raw[0] + self.t_inv.m32 * raw[1] + self.t_inv.m33 * raw[2];
        m
    }
}

#[cfg(test)]
mod test {
    use geo::*;
    use super::*;
    #[test]
    fn test_affine() {
        let src = vec![Pointf::new(0f32,0f32), Pointf::new(1f32, 0f32), Pointf::new(0f32, 1f32)];
        let dst = vec![Pointf::new(1f32,0f32), Pointf::new(1f32, 1f32), Pointf::new(0f32, 0f32)];
        let aff1 = Affine2D::affine_from_points(&src, &dst).unwrap();
        assert_eq!(aff1.map_point(src[0].clone()), dst[0]);
        assert_eq!(aff1.map_point_inv(dst[0].clone()), src[0]);
    }

    fn test_noreflect() {
        let src = vec![Pointf::new(0f32, 0f32), Pointf::new(1f32, 0f32)];
        let dst = vec![Pointf::new(0f32, 0f32), Pointf::new(0f32, 2f32)];
        let aff = Affine2D::nonreflect_similarity_from_points(&src, &dst).unwrap();
        assert_eq!(aff.map_point(src[1].clone()), dst[1]);
        assert_eq!(aff.map_point_inv(dst[1].clone()), src[1]);

        let pt = Pointf::new(2f32, 0f32);
        assert_eq!(aff.map_point(pt), Pointf::new(0f32, 4f32));
    }
}

