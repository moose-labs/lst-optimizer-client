# s program (for testing)

Build by `cargo build-sbf --features testing`

## Notes

### Hacks

1. Epoch checking bypass in `lido_calculator` and `spl_calculator` based

    ```rust
    impl LidoCalc {
        #[cfg(feature = "testing")]
        pub const fn verify_pool_updated_for_this_epoch(
            &self,
            this_epoch: u64,
        ) -> Result<(), LidoCalculatorError> {
            Ok(())
        }

        #[cfg(not(feature = "testing"))]
        pub const fn verify_pool_updated_for_this_epoch(
            &self,
            this_epoch: u64,
        ) -> Result<(), LidoCalculatorError> {
            // The original code checks computed_in_epoch >= this_epoch,
            // but if computed_in_epoch is somehow > this_epoch there's probably
            // something weird going on, so we should just fail too
            if self.computed_in_epoch == this_epoch {
                Ok(())
            } else {
                Err(LidoCalculatorError::ExchangeRateNotUpdatedInThisEpoch)
            }
        }
    }
    ```

    ```rust
    impl SplStakePoolCalc {
        #[cfg(feature = "testing")]
        pub const fn verify_pool_updated_for_this_epoch(
            &self,
            this_epoch: u64,
        ) -> Result<(), SplCalculatorError> {
            Ok(())
        }

        #[cfg(not(feature = "testing"))]
        pub const fn verify_pool_updated_for_this_epoch(
            &self,
            this_epoch: u64,
        ) -> Result<(), SplCalculatorError> {
            if self.last_update_epoch == this_epoch {
                Ok(())
            } else {
                Err(SplCalculatorError::PoolNotUpdated)
            }
        }
    }
    ```
