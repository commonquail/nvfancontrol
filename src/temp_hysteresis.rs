#[derive(Debug)]
pub struct TemperatureHysteron {
    offset: i32,
    alpha: Option<i32>,
    beta: i32,
}

impl TemperatureHysteron {

    pub fn with_offset(degrees: i32) -> Result<Self, String> {
        if degrees < 0 {
            return Err("Degree offset must be non-negative.".to_owned());
        }

        let new = Self {
            offset: degrees,
            beta: 0,
            alpha: None,
        };

        Ok(new)
    }

    pub fn get_temp(&mut self, temp: i32) -> i32 {
        debug!("Hysteresis: temp={}, temp_hysteron={:?}", temp, self);

        if temp == self.beta  {
            // This is the most likely scenario when increasing temp, but
            // also prevents repeat "lower beta" when "x <= \alpha"
            debug!("Hysteresis: stable short-circuit");
            temp
        } else if temp > self.beta  {
            // In Preisach, "x >= \beta". Here "x > \beta" yields same result
            // so skip that redundant case.

            // Always respond to increasing temperature.
            self.beta = temp;

            self.alpha = Some(self.beta - self.offset);
            debug!("Hysteresis: raise beta: {}", self.beta);
            temp
        } else if self.alpha.is_none() {
            self.beta = temp;
            debug!("Hysteresis: lower beta: {}, disabled", self.beta);
            temp
        } else if temp <= self.alpha.unwrap() {
            self.beta = temp;
            self.alpha = None;
            debug!("Hysteresis: lower beta: {}, disable", self.beta);
            temp
        } else {
            // Most likely scenario after slowing down fans again...?
            debug!("Hysteresis: stable");
            // We could return beta in every branch but that would misrepresent
            // the model.
            self.beta
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_instantiate_with_zero_offset() {
        let hysteron = TemperatureHysteron::with_offset(0);
        assert!(hysteron.is_ok());
    }

    #[test]
    fn cannot_instantiate_with_negative_offset() {
        let hysteron = TemperatureHysteron::with_offset(-1);
        assert!(hysteron.is_err());
    }

    #[test]
    fn offset_0_is_linear() {
        let mut hysteron = with_offset(0);

        let init = hysteron.get_temp(0);
        assert_eq!(init, 0, "{:?}", hysteron);

        let increase = hysteron.get_temp(1);
        assert_eq!(increase, 1, "{:?}", hysteron);

        let decrease = hysteron.get_temp(0);
        assert_eq!(decrease, 0, "{:?}", hysteron);
    }

    #[test]
    fn offset_1_is_effectively_linear() {
        // While we use >= and <=, an offset of 1 will always yield y(x) = x.
        // There is no window for x to fall into.

        let mut hysteron = with_offset(1);

        let init = hysteron.get_temp(0);
        assert_eq!(init, 0, "{:?}", hysteron);

        let increase = hysteron.get_temp(1);
        assert_eq!(increase, 1, "{:?}", hysteron);

        let decrease = hysteron.get_temp(0);
        assert_eq!(decrease, 0, "{:?}", hysteron);
    }

    #[test]
    fn offset_greater_equal_2_is_nonlinear() {
        let some_offset = 2;
        let some_beta = 10;

        let mut hysteron = with_offset(some_offset);

        hysteron.get_temp(some_beta);

        let stable_lower_bound = some_beta - some_offset + 1;
        let stable_decrease = hysteron.get_temp(stable_lower_bound);
        assert_eq!(stable_decrease, some_beta, "{:?}", hysteron);

        let lower_cutoff_point = some_beta - some_offset;
        let decrease = hysteron.get_temp(lower_cutoff_point);
        assert_eq!(decrease, lower_cutoff_point, "{:?}", hysteron);
    }

    #[test]
    fn decrease_is_linear_after_hysteresis() {
        // Don't make a new lag window every time the temperature decreases.
        // That would mean fan speed decreases very slowly when temperature
        // decreases and that's not what we want. It's only the constant
        // switching we want to avoid. Instead, when temperature decreases
        // continually just follow along.

        let some_offset = 2;
        let some_beta = 10;

        let mut hysteron = with_offset(some_offset);

        hysteron.get_temp(some_beta);

        let lower_cutoff_point = some_beta - some_offset;
        hysteron.get_temp(lower_cutoff_point);

        let cutoff_minus_1 = lower_cutoff_point - 1;
        let decrease = hysteron.get_temp(cutoff_minus_1);
        assert_eq!(decrease, cutoff_minus_1, "{:?}", hysteron);
    }

    fn with_offset(some_offset: i32) -> TemperatureHysteron {
        TemperatureHysteron::with_offset(some_offset).unwrap()
    }
}
