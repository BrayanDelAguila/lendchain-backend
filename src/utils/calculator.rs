//! Loan financial calculator utilities.

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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 0.01;

    #[test]
    fn test_monthly_payment_known_value() {
        // $1000 at 5% annual rate over 12 months ≈ $85.61
        let payment = monthly_payment(1000.0, 0.05, 12);
        assert!(
            (payment - 85.61).abs() < EPSILON,
            "Expected ≈85.61, got {:.4}",
            payment
        );
    }

    #[test]
    fn test_monthly_payment_zero_rate() {
        let payment = monthly_payment(1200.0, 0.0, 12);
        assert!(
            (payment - 100.0).abs() < EPSILON,
            "Zero-rate payment should equal principal/months, got {:.4}",
            payment
        );
    }

    #[test]
    fn test_schedule_length() {
        let schedule = amortisation_schedule(1000.0, 0.05, 24);
        assert_eq!(
            schedule.len(),
            24,
            "Schedule must have exactly term_months rows"
        );
    }

    #[test]
    fn test_schedule_first_row_adds_up() {
        let schedule = amortisation_schedule(1000.0, 0.12, 12);
        let row = &schedule[0];
        let reconstructed = row.principal + row.interest;
        assert!(
            (reconstructed - row.payment).abs() < EPSILON,
            "principal + interest should equal payment; got {:.4} + {:.4} = {:.4}, payment = {:.4}",
            row.principal,
            row.interest,
            reconstructed,
            row.payment
        );
    }

    #[test]
    fn test_schedule_final_balance_near_zero() {
        let schedule = amortisation_schedule(5000.0, 0.08, 36);
        let last = schedule.last().expect("schedule should not be empty");
        assert!(
            last.remaining_balance < EPSILON,
            "Final balance should be near zero, got {:.6}",
            last.remaining_balance
        );
    }

    #[test]
    fn test_schedule_total_payment_consistency() {
        let principal = 1000.0;
        let schedule = amortisation_schedule(principal, 0.05, 12);
        let total_principal: f64 = schedule.iter().map(|r| r.principal).sum();
        let total_interest: f64 = schedule.iter().map(|r| r.interest).sum();
        let total_paid: f64 = schedule.iter().map(|r| r.payment).sum();
        assert!(
            (total_principal + total_interest - total_paid).abs() < EPSILON,
            "sum(principal) + sum(interest) must equal sum(payment)"
        );
        assert!(
            (total_principal - principal).abs() < EPSILON,
            "Total principal repaid should equal the original principal"
        );
    }

    #[test]
    fn test_schedule_single_month() {
        let principal = 1000.0;
        let annual_rate = 0.12;
        let schedule = amortisation_schedule(principal, annual_rate, 1);
        assert_eq!(schedule.len(), 1);
        let expected_interest = principal * (annual_rate / 12.0);
        let expected_payment = principal + expected_interest;
        assert!(
            (schedule[0].payment - expected_payment).abs() < EPSILON,
            "Single-month payment should be principal + one month of interest"
        );
        assert!(schedule[0].remaining_balance < EPSILON);
    }
}
