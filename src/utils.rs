// 1500km - km off
pub fn calculate_score(kms: f64) -> u64 {
    let kilometers = kms.floor() as u64;
    if kilometers > 1500 {
        return 0;
    }

    (1500 - kilometers).div_ceil(4)
}
