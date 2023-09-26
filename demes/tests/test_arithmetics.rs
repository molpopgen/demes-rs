use demes::{CloningRate, DemeSize, GenerationTime, MigrationRate, Proportion, SelfingRate, Time};

macro_rules! test_newtype_arithmetics {
    ($type: ty, $fn_name: ident, $a:expr,$b:expr) => {
        #[test]
        fn $fn_name() {
            let a = $a;
            let b = $b;
            let a_type = <$type>::try_from(a).unwrap();
            let b_type = <$type>::try_from(b).unwrap();
            let res_add = <$type>::try_from(a + b).unwrap();
            let res_sub_ok = <$type>::try_from(a - b).unwrap();
            let res_mul = <$type>::try_from(a * b).unwrap();
            let res_div = <$type>::try_from(b / a).unwrap();

            assert_eq!(
                (a_type + b_type).unwrap(),
                <$type>::try_from(res_add).unwrap()
            );
            assert_eq!((a_type + b).unwrap(), <$type>::try_from(res_add).unwrap());
            assert_eq!((a + b_type).unwrap(), <$type>::try_from(res_add).unwrap());

            assert_eq!(
                (a_type - b_type).unwrap(),
                <$type>::try_from(res_sub_ok).unwrap()
            );
            assert_eq!(
                (a_type - b).unwrap(),
                <$type>::try_from(res_sub_ok).unwrap()
            );
            assert_eq!(
                (a - b_type).unwrap(),
                <$type>::try_from(res_sub_ok).unwrap()
            );
            assert!((b_type - a_type).is_none());
            assert!((b_type - a).is_none());
            assert!((b - a_type).is_none());

            assert_eq!(
                (a_type * b_type).unwrap(),
                <$type>::try_from(res_mul).unwrap()
            );
            assert_eq!((a_type * b).unwrap(), <$type>::try_from(res_mul).unwrap());
            assert_eq!((a * b_type).unwrap(), <$type>::try_from(res_mul).unwrap());

            assert_eq!(
                (b_type / a_type).unwrap(),
                <$type>::try_from(res_div).unwrap()
            );
            assert_eq!((b_type / a).unwrap(), <$type>::try_from(res_div).unwrap());
            assert_eq!((b / a_type).unwrap(), <$type>::try_from(res_div).unwrap());
        }
    };
}

test_newtype_arithmetics!(DemeSize, test_arithmetics_demesize, 10., 9.);
test_newtype_arithmetics!(Time, test_arithmetics_time, 10., 9.);
test_newtype_arithmetics!(GenerationTime, test_arithmetics_generationtime, 10., 9.);
test_newtype_arithmetics!(Proportion, test_arithmetics_proportion, 0.4, 0.3);
test_newtype_arithmetics!(CloningRate, test_arithmetics_cloningrate, 0.4, 0.3);
test_newtype_arithmetics!(MigrationRate, test_arithmetics_migrationrate, 0.4, 0.3);
test_newtype_arithmetics!(SelfingRate, test_arithmetics_selfingrate, 0.4, 0.3);
