//! Temporal train/validation/test splits with anti-leakage guarantees.

use serde::{Deserialize, Serialize};
use sp_kernel::TimeWindow;

use crate::domain::error::{DatasetError, Result};

/// A leakage-free temporal split for model training.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSplit {
    train: TimeWindow,
    val: Option<TimeWindow>,
    test: TimeWindow,
}

impl DataSplit {
    /// Creates a split, rejecting any overlapping windows (anti-leakage).
    pub fn new(train: TimeWindow, val: Option<TimeWindow>, test: TimeWindow) -> Result<Self> {
        check_no_overlap(&train, "train", &test, "test")?;
        if let Some(ref validation) = val {
            check_no_overlap(&train, "train", validation, "val")?;
            check_no_overlap(validation, "val", &test, "test")?;
        }
        Ok(Self { train, val, test })
    }

    /// Training window.
    pub fn train(&self) -> TimeWindow {
        self.train
    }

    /// Optional validation window.
    pub fn val(&self) -> Option<TimeWindow> {
        self.val
    }

    /// Test window.
    pub fn test(&self) -> TimeWindow {
        self.test
    }

    /// Returns all windows with their split names.
    pub fn windows(&self) -> Vec<(&'static str, TimeWindow)> {
        let mut windows = vec![("train", self.train)];
        if let Some(validation) = self.val {
            windows.push(("val", validation));
        }
        windows.push(("test", self.test));
        windows
    }
}

fn check_no_overlap(
    a: &TimeWindow,
    a_name: &'static str,
    b: &TimeWindow,
    b_name: &'static str,
) -> Result<()> {
    if a.overlaps(b) {
        return Err(DatasetError::WindowOverlap {
            a: a_name,
            b: b_name,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp;

    fn ts(seconds: i64) -> Timestamp {
        Timestamp::from_second(seconds).unwrap()
    }

    fn window(start: i64, end: i64) -> TimeWindow {
        TimeWindow::new(ts(start), ts(end)).unwrap()
    }

    #[test]
    fn rejects_overlapping_train_and_test() {
        let train = window(100, 250);
        let test = window(200, 300);
        let err = DataSplit::new(train, None, test).unwrap_err();
        assert_eq!(
            err,
            DatasetError::WindowOverlap {
                a: "train",
                b: "test"
            }
        );
    }

    #[test]
    fn accepts_disjoint_windows_without_val() {
        let train = window(100, 200);
        let test = window(200, 300);
        let split = DataSplit::new(train, None, test).unwrap();
        assert_eq!(split.train(), train);
        assert_eq!(split.val(), None);
        assert_eq!(split.test(), test);
    }

    #[test]
    fn accepts_val_in_the_middle() {
        let train = window(100, 200);
        let val = window(200, 300);
        let test = window(300, 400);
        let split = DataSplit::new(train, Some(val), test).unwrap();
        assert_eq!(split.val(), Some(val));
        assert_eq!(split.windows().len(), 3);
    }
}
