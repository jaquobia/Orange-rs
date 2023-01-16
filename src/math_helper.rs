pub mod angle {
    const FRAC_PI_180: f64 = std::f64::consts::PI / 180.0;
    const FRAC_180_PI: f64 = 180.0 / std::f64::consts::PI;

    #[derive(Debug)]
    pub struct Rad<T>(pub T);
    #[derive(Debug)]
    pub struct Deg<T>(pub T);

    impl From<Deg<f32>> for Rad<f32> {
        fn from(item: Deg<f32>) -> Self {
            Self((item.0 as f64 * FRAC_PI_180) as f32)
        }
    }
    impl From<Deg<f64>> for Rad<f64> {
        fn from(item: Deg<f64>) -> Self {
            Self(item.0 * FRAC_PI_180)
        }
    }
    impl From<Rad<f32>> for Deg<f32> {
        fn from(item: Rad<f32>) -> Self {
            Self((item.0 as f64 * FRAC_180_PI) as f32)
        }
    }
    impl From<Rad<f64>> for Deg<f64> {
        fn from(item: Rad<f64>) -> Self {
            Self(item.0 * FRAC_180_PI)
        }
    }
}
