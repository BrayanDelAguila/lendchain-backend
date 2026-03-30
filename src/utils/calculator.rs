/// Loan financial calculator utilities.

/// Calculate the fixed monthly payment using the standard amortisation formula:
///   M = P * [r(1+r)^n] / [(1+r)^n - 1]
/// where r = monthly_rate, n = term_months
pub fn monthly_payment(principal: f64, annual_rate: f64, term_months: u32) -> f64 {
    if annual_rate == 0.0 {
        return principal / term_months as f64;
    }
    let r = annual_rate / 12.0;
    let n = term_months as f64;
    let factor = (1.0 + r).powf(n);
    principal * r * factor / (factor - 1.0)
}

/// Generate the full amortisation schedule for a loan.
pub fn amortisation_schedule(
    principal: f64,
    annual_rate: f64,
    term_months: u32,
) -> Vec<AmortisationRow> {
    let payment = monthly_payment(principal, annual_rate, term_months);
    let monthly_rate = annual_rate / 12.0;
    let mut balance = principal;
    let mut schedule = Vec::with_capacity(term_months as usize);

    for month in 1..=term_months {
        let interest = balance * monthly_rate;
        let principal_part = (payment - interest).max(0.0);
        balance -= principal_part;

        schedule.push(AmortisationRow {
            payment_number: month,
            payment,
            principal: principal_part,
            interest,
            remaining_balance: balance.max(0.0),
        });
    }
    schedule
}

#[derive(Debug)]
pub struct AmortisationRow {
    pub payment_number: u32,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}
