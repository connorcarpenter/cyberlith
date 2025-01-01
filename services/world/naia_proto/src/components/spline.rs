use std::iter::once;

use itertools::Itertools;

use math::Vec2;

#[derive(Clone, Debug)]
struct CubicSegment {
    coeff: [Vec2; 4],
}

impl CubicSegment {
    /// Instantaneous position of a point at parametric value `t`.
    fn position(&self, t: f32) -> Vec2 {
        let [a, b, c, d] = self.coeff;
        // Evaluate `a + bt + ct^2 + dt^3`, avoiding exponentiation
        a + (b + (c + d * t) * t) * t
    }

    /// Instantaneous velocity of a point at parametric value `t`.
    fn velocity(&self, t: f32) -> Vec2 {
        let [_, b, c, d] = self.coeff;
        // Evaluate the derivative, which is `b + 2ct + 3dt^2`, avoiding exponentiation
        b + (c * 2.0 + d * 3.0 * t) * t
    }

    /// Instantaneous acceleration of a point at parametric value `t`.
    fn acceleration(&self, t: f32) -> Vec2 {
        let [_, _, c, d] = self.coeff;
        // Evaluate the second derivative, which is `2c + 6dt`
        c * 2.0 + d * 6.0 * t
    }

    /// Calculate polynomial coefficients for the cubic curve using a characteristic matrix.
    #[inline]
    fn coefficients(p: [Vec2; 4], char_matrix: [[f32; 4]; 4]) -> Self {
        let [c0, c1, c2, c3] = char_matrix;
        // These are the polynomial coefficients, computed by multiplying the characteristic
        // matrix by the point matrix.
        let coeff = [
            p[0] * c0[0] + p[1] * c0[1] + p[2] * c0[2] + p[3] * c0[3],
            p[0] * c1[0] + p[1] * c1[1] + p[2] * c1[2] + p[3] * c1[3],
            p[0] * c2[0] + p[1] * c2[1] + p[2] * c2[2] + p[3] * c2[3],
            p[0] * c3[0] + p[1] * c3[1] + p[2] * c3[2] + p[3] * c3[3],
        ];
        Self { coeff }
    }
}

/// A piecewise spline path, each segment is [i..i+1] in param.
#[derive(Clone, Debug)]
pub struct SplinePath {
    pub segments: Vec<CubicSegment>,
}

impl SplinePath {

    /// The characteristic matrix for this spline construction.
    ///
    /// Each row of this matrix expresses the coefficients of a [`CubicSegment`] as a linear
    /// combination of four consecutive control points.
    #[inline]
    fn char_matrix() -> [[f32; 4]; 4] {
        let s = 0.5;
        [
            [0., 1., 0., 0.],
            [-s, 0., s, 0.],
            [2. * s, s - 3., 3. - 2. * s, -s],
            [-s, 2. - s, s - 2., s],
        ]
    }

    pub fn new(control_points: &[Vec2]) -> Self {
        let length = control_points.len();

        // Early return to avoid accessing an invalid index
        if length < 2 {
            panic!("A spline path must have at least two control points.");
        }

        // Extend the list of control points by mirroring the last second-to-last control points on each end;
        // this allows tangents for the endpoints to be provided, and the overall effect is that the tangent
        // at an endpoint is proportional to twice the vector between it and its adjacent control point.
        //
        // The expression used here is P_{-1} := P_0 - (P_1 - P_0) = 2P_0 - P_1. (Analogously at the other end.)
        let mirrored_first = control_points[0] * 2. - control_points[1];
        let mirrored_last = control_points[length - 1] * 2. - control_points[length - 2];
        let extended_control_points = once(&mirrored_first)
            .chain(control_points.iter())
            .chain(once(&mirrored_last));

        let segments = extended_control_points
            .tuple_windows()
            .map(|(&p0, &p1, &p2, &p3)| {
                CubicSegment::coefficients([p0, p1, p2, p3], Self::char_matrix())
            })
            .collect_vec();

        Self {
            segments,
        }
    }

    /// Compute the position of a point on the cubic curve at the parametric value `t`.
    ///
    /// Note that `t` varies from `0..=(n_points - 3)`.
    #[inline]
    pub fn position(&self, t: f32) -> Vec2 {
        let (segment, t) = self.segment(t);
        segment.position(t)
    }

    /// Compute the first derivative with respect to t at `t`. This is the instantaneous velocity of
    /// a point on the cubic curve at `t`.
    ///
    /// Note that `t` varies from `0..=(n_points - 3)`.
    #[inline]
    pub fn velocity(&self, t: f32) -> Vec2 {
        let (segment, t) = self.segment(t);
        segment.velocity(t)
    }

    /// Compute the second derivative with respect to t at `t`. This is the instantaneous
    /// acceleration of a point on the cubic curve at `t`.
    ///
    /// Note that `t` varies from `0..=(n_points - 3)`.
    #[inline]
    pub fn acceleration(&self, t: f32) -> Vec2 {
        let (segment, t) = self.segment(t);
        segment.acceleration(t)
    }

    /// The list of segments contained in this `CubicCurve`.
    ///
    /// This spline's global `t` value is equal to how many segments it has.
    ///
    /// All method accepting `t` on `CubicCurve` depends on the global `t`.
    /// When sampling over the entire curve, you should either use one of the
    /// `iter_*` methods or account for the segment count using `curve.segments().len()`.
    #[inline]
    pub fn segments(&self) -> &[CubicSegment] {
        &self.segments
    }

    #[inline]
    /// Adds a segment to the curve
    pub fn push_segment(&mut self, segment: CubicSegment) {
        self.segments.push(segment);
    }

    /// Returns the [`CubicSegment`] and local `t` value given a spline's global `t` value.
    #[inline]
    fn segment(&self, t: f32) -> (&CubicSegment, f32) {
        if self.segments.len() == 1 {
            (&self.segments[0], t)
        } else {
            let i = (t.floor() as usize).clamp(0, self.segments.len() - 1);
            (&self.segments[i], t - i as f32)
        }
    }
}