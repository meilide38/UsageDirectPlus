use jni::objects::{JClass, JIntArray, JLongArray};
use jni::sys::{jint, jlongArray};
use jni::JNIEnv;

/// Sum each interval duration into the slot identified by its package index.
/// Uses wrapping arithmetic to match Kotlin/JVM Long addition semantics exactly.
pub fn aggregate_durations(
    begin_times: &[i64],
    end_times: &[i64],
    app_indexes: &[i32],
    app_count: usize,
) -> Vec<i64> {
    let mut totals = vec![0_i64; app_count];
    for ((begin, end), app_index) in begin_times
        .iter()
        .zip(end_times.iter())
        .zip(app_indexes.iter())
    {
        if let Ok(index) = usize::try_from(*app_index) {
            if index < totals.len() {
                totals[index] = totals[index].wrapping_add(end.wrapping_sub(*begin));
            }
        }
    }
    totals
}

#[no_mangle]
pub extern "system" fn Java_godau_fynn_usagedirectplus_wrapper_RustUsageCore_nativeAggregate(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    begin_array: JLongArray<'_>,
    end_array: JLongArray<'_>,
    app_index_array: JIntArray<'_>,
    app_count: jint,
) -> jlongArray {
    let result = (|| -> jni::errors::Result<jlongArray> {
        let begin_len = env.get_array_length(&begin_array)? as usize;
        let end_len = env.get_array_length(&end_array)? as usize;
        let index_len = env.get_array_length(&app_index_array)? as usize;
        if begin_len != end_len || begin_len != index_len || app_count < 0 {
            return Err(jni::errors::Error::NullPtr("invalid aggregation input"));
        }

        let mut begins = vec![0_i64; begin_len];
        let mut ends = vec![0_i64; end_len];
        let mut indexes = vec![0_i32; index_len];
        env.get_long_array_region(&begin_array, 0, &mut begins)?;
        env.get_long_array_region(&end_array, 0, &mut ends)?;
        env.get_int_array_region(&app_index_array, 0, &mut indexes)?;

        let totals = aggregate_durations(&begins, &ends, &indexes, app_count as usize);
        let output = env.new_long_array(totals.len() as i32)?;
        env.set_long_array_region(&output, 0, &totals)?;
        Ok(output.into_raw())
    })();

    match result {
        Ok(array) => array,
        Err(_) => {
            let _ = env.exception_clear();
            std::ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::aggregate_durations;

    #[test]
    fn sums_multiple_intervals_per_app_without_losing_order() {
        let totals = aggregate_durations(&[10, 50, 100], &[30, 80, 160], &[0, 1, 0], 2);
        assert_eq!(totals, vec![80, 30]);
    }

    #[test]
    fn ignores_invalid_package_indexes() {
        let totals = aggregate_durations(&[0, 10, 20], &[5, 20, 30], &[0, -1, 4], 1);
        assert_eq!(totals, vec![5]);
    }

    #[test]
    fn empty_input_returns_zero_totals_for_requested_apps() {
        assert_eq!(aggregate_durations(&[], &[], &[], 3), vec![0, 0, 0]);
    }

    #[test]
    fn preserves_wrapping_long_arithmetic() {
        let totals = aggregate_durations(&[0, 0], &[i64::MAX, 1], &[0, 0], 1);
        assert_eq!(totals, vec![i64::MIN]);
    }
}
