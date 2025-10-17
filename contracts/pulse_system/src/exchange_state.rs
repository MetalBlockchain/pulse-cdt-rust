use pulse_cdt::core::{Asset, Symbol};

#[inline(always)]
pub fn get_bancor_input(out_reserve: i64, inp_reserve: i64, out: i64) -> i64 {
    let ob = out_reserve as f64;
    let ib = inp_reserve as f64;

    if ob <= out as f64 {
        return 0; // avoid divide-by-zero or negative denominator
    }

    let mut inp = (ib * (out as f64)) / (ob - (out as f64));
    if inp < 0.0 {
        inp = 0.0;
    }

    inp as i64
}

pub fn get_bancor_output(inp_reserve: i64, out_reserve: i64, inp: i64) -> i64 {
    let ib = inp_reserve as f64;
    let ob = out_reserve as f64;
    let inn = inp as f64;

    let mut out = ((inn * ob) / (ib + inn)) as i64;

    if out < 0 {
        out = 0;
    }

    out
}