use bulletproofs::r1cs::{ConstraintSystem, LinearCombination, Variable};
use curve25519_dalek::scalar::Scalar;

pub fn score_gadget<CS: ConstraintSystem>(
    cs: &mut CS,
    d: LinearCombination,
    y: LinearCombination,
    y_inv: LinearCombination,
    q: LinearCombination,
) {
    let one = Scalar::one();

    // check that Yinv * Y = 1
    let (_, _, one_var) = cs.multiply(y, y_inv.clone());
    cs.constrain(one_var - one);

    // Q = F(d,Y)
    let (_, _, q_var) = cs.multiply(d, y_inv);
    cs.constrain(q - q_var);
}

pub fn one_of_many_gadget<CS: ConstraintSystem>(
    cs: &mut CS,
    x: LinearCombination,          // private: our item x
    toggle: Vec<Variable>,         // private: binary list indicating it is somewhere in list
    items: Vec<LinearCombination>, // public list
) {
    let toggle_len = toggle.len();

    // ensure every item in toggle is binary
    for i in toggle.iter() {
        boolean_gadget(cs, i.clone().into());
    }

    // toggle_sum[i] = toggle_sum(i-1) + toggle(i)
    let mut toggle_sum: Vec<LinearCombination> = Vec::with_capacity(toggle_len);
    toggle_sum.push(toggle[0].clone().into());
    for i in 1..toggle_len {
        let prev_toggle_sum = toggle_sum[i - 1].clone();
        let curr_toggle = toggle[i].clone();

        toggle_sum.push(prev_toggle_sum + (curr_toggle.clone()));
    }

    // ensure sum of toggles = 1
    for i in 1..toggle_len {
        let prev_toggle_sum = toggle_sum[i - 1].clone();
        let curr_toggle = toggle[i].clone();
        let curr_toggle_sum = toggle_sum[i].clone();

        toggle_sum[i] = toggle_sum[i - 1].clone() + (toggle[i].clone());

        cs.constrain(prev_toggle_sum + (curr_toggle) - (curr_toggle_sum));
    }
    let one: Scalar = Scalar::one();
    let last_item = toggle_sum[toggle_len - 1].clone();
    cs.constrain(last_item - one);

    // now check if item is in list
    // item[i] * toggle[i] = toggle[i] * our item (x)
    for i in 0..toggle_len {
        let (_, _, left) = cs.multiply(items[i].clone(), toggle[i].clone().into());
        let (_, _, right) = cs.multiply(toggle[i].clone().into(), x.clone());
        cs.constrain(left - right);
    }
}

pub fn boolean_gadget<CS: ConstraintSystem>(cs: &mut CS, a1: LinearCombination) {
    // a *(1-a) = 0
    let a = a1.clone();
    let one: LinearCombination = Scalar::one().into();
    let (_, _, c_var) = cs.multiply(a, one - a1);
    cs.constrain(c_var.into());
}
