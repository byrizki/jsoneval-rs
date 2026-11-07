/// Represents a contiguous 2D float tensor of dimension [rows × cols] stored in row-major order.
#[derive(Debug, Clone)]
pub struct Tensor2D {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Tensor2D {
    /// Creates a new Tensor2D with dimensions [rows, cols] filled with 0.0
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    /// Read cell at (row, col)
    #[inline(always)]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    /// Set cell at (row, col)
    #[inline(always)]
    pub fn set(&mut self, row: usize, col: usize, val: f64) {
        self.data[row * self.cols + col] = val;
    }

    /// Read raw slice
    #[inline(always)]
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }

    /// Read mutable raw slice
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.data
    }

    /// Copy a column into a separate vector
    pub fn extract_col(&self, col: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            out.push(self.get(r, col));
        }
        out
    }

    /// Write an entire column from a slice
    pub fn set_col(&mut self, col: usize, src: &[f64]) {
        let limit = self.rows.min(src.len());
        for r in 0..limit {
            self.set(r, col, src[r]);
        }
    }
}

/// Columnar vector operations operating across all time steps (rows)
#[derive(Debug, Clone)]
pub enum VectorOp {
    /// dest[t] = constant
    VectorFill { constant: f64, dest: usize },

    /// dest[t] = src[t]
    VectorCopy { src: usize, dest: usize },

    /// dest[t] = src[t] + constant
    VectorAddConst { src: usize, constant: f64, dest: usize },

    /// dest[t] = src[t] * factor
    VectorScale { src: usize, factor: f64, dest: usize },

    /// dest[t] = src1[t] + src2[t]
    VectorAdd { src1: usize, src2: usize, dest: usize },

    /// dest[t] = src1[t] - src2[t]
    VectorSub { src1: usize, src2: usize, dest: usize },

    /// dest[t] = src1[t] * src2[t]
    VectorMul { src1: usize, src2: usize, dest: usize },

    /// dest[t] = if src2[t] != 0 { src1[t] / src2[t] } else { 0.0 }
    VectorDiv { src1: usize, src2: usize, dest: usize },

    /// dest[t] = 1.0 - src[t]
    VectorOneMinus { src: usize, dest: usize },

    /// Cumulative product scan for decrement/survival probabilities:
    /// dest[0] = 1.0; dest[t] = dest[t-1] * (1.0 - decrement[t-1])
    CumulativeProductScan { decrement_col: usize, dest: usize },

    /// Backward discount scan for Expected Present Value (EPV):
    /// When timing == End:   epv[t] = (epv[t+1] + ecf[t]) / (1 + rate[t])
    /// When timing == Start: epv[t] = epv[t+1] / (1 + rate[t]) + ecf[t]
    ReverseDiscountScan {
        ecf_col: usize,
        rate_col: usize,
        dest: usize,
        is_start_timing: bool,
    },

    /// Ternary condition mask: dest[t] = if mask[t] > 0.5 { src_true[t] } else { src_false[t] }
    VectorWhere {
        mask_col: usize,
        src_true_col: usize,
        src_false_col: usize,
        dest: usize,
    },
}

impl VectorOp {
    /// Execute this vector operation directly on a Tensor2D across all rows
    pub fn execute(&self, tensor: &mut Tensor2D) {
        let rows = tensor.rows;
        let cols = tensor.cols;

        match self {
            VectorOp::VectorFill { constant, dest } => {
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = *constant;
                }
            }
            VectorOp::VectorCopy { src, dest } => {
                let s = *src;
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s];
                }
            }
            VectorOp::VectorAddConst { src, constant, dest } => {
                let s = *src;
                let d = *dest;
                let c = *constant;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s] + c;
                }
            }
            VectorOp::VectorScale { src, factor, dest } => {
                let s = *src;
                let d = *dest;
                let f = *factor;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s] * f;
                }
            }
            VectorOp::VectorAdd { src1, src2, dest } => {
                let s1 = *src1;
                let s2 = *src2;
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s1] + tensor.data[r * cols + s2];
                }
            }
            VectorOp::VectorSub { src1, src2, dest } => {
                let s1 = *src1;
                let s2 = *src2;
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s1] - tensor.data[r * cols + s2];
                }
            }
            VectorOp::VectorMul { src1, src2, dest } => {
                let s1 = *src1;
                let s2 = *src2;
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = tensor.data[r * cols + s1] * tensor.data[r * cols + s2];
                }
            }
            VectorOp::VectorDiv { src1, src2, dest } => {
                let s1 = *src1;
                let s2 = *src2;
                let d = *dest;
                for r in 0..rows {
                    let denom = tensor.data[r * cols + s2];
                    tensor.data[r * cols + d] = if denom.abs() > 1e-12 {
                        tensor.data[r * cols + s1] / denom
                    } else {
                        0.0
                    };
                }
            }
            VectorOp::VectorOneMinus { src, dest } => {
                let s = *src;
                let d = *dest;
                for r in 0..rows {
                    tensor.data[r * cols + d] = 1.0 - tensor.data[r * cols + s];
                }
            }
            VectorOp::CumulativeProductScan { decrement_col, dest } => {
                let dec = *decrement_col;
                let d = *dest;
                if rows == 0 {
                    return;
                }
                tensor.data[0 * cols + d] = 1.0;
                for r in 1..rows {
                    let prev_val = tensor.data[(r - 1) * cols + d];
                    let q = tensor.data[(r - 1) * cols + dec];
                    tensor.data[r * cols + d] = prev_val * (1.0 - q);
                }
            }
            VectorOp::ReverseDiscountScan {
                ecf_col,
                rate_col,
                dest,
                is_start_timing,
            } => {
                let ecf = *ecf_col;
                let rate = *rate_col;
                let d = *dest;
                let is_start = *is_start_timing;

                if rows == 0 {
                    return;
                }

                // Initialize terminal value (after the end of the projection, value is 0.0)
                let mut next_epv = 0.0;

                // Step backward from t = rows - 1 down to 0
                for r in (0..rows).rev() {
                    let r_rate = tensor.data[r * cols + rate];
                    let r_ecf = tensor.data[r * cols + ecf];
                    let discount_factor = 1.0 + r_rate;

                    let epv = if is_start {
                        // Annuity-due (e.g. premium paid at start of period):
                        // epv[t] = epv[t+1] / (1 + r) + ecf[t]
                        (next_epv / discount_factor) + r_ecf
                    } else {
                        // Annuity-immediate (e.g. claim paid at end of period):
                        // epv[t] = (epv[t+1] + ecf[t]) / (1 + r)
                        (next_epv + r_ecf) / discount_factor
                    };

                    tensor.data[r * cols + d] = epv;
                    next_epv = epv;
                }
            }
            VectorOp::VectorWhere {
                mask_col,
                src_true_col,
                src_false_col,
                dest,
            } => {
                let m = *mask_col;
                let st = *src_true_col;
                let sf = *src_false_col;
                let d = *dest;
                for r in 0..rows {
                    let is_true = tensor.data[r * cols + m] > 0.5;
                    tensor.data[r * cols + d] = if is_true {
                        tensor.data[r * cols + st]
                    } else {
                        tensor.data[r * cols + sf]
                    };
                }
            }
        }
    }
}
