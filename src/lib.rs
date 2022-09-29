//! The library behind `vihr` command line time tracking app.

#![warn(missing_docs)]

use chrono::{Duration, Local, NaiveDateTime};

use std::collections::{HashMap, hash_map::Entry};
use std::fmt::Display;

/// An error with a message intended to be shown to the user.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Caused by trying to access a category that doesn't exist.
    CategoryDoesntExist(String),
    /// Caused by trying to create a category that already exists.
    CategoryExists(String),
    /// Caused by trying to access a `TimeUsage` that doesn't exist.
    TimeUsageDoesntExist(u64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CategoryExists(cat) => {
                write!(f, "Category {} already exists.", cat)
            }
            Self::CategoryDoesntExist(cat) => {
                write!(f, "Category {} doesn't exist.", cat)
            }
            Self::TimeUsageDoesntExist(id) => {
                write!(f, "Time Usage with the id {} doesn't exist.", id)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Alias for `Result` with the error type of `crate::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Defines a time span when time was spent on doing something.
/// `TimeUsage`s are sorted by their starting time.
pub struct TimeUsage {
    /// The starting point of the `TimeUsage`.
    pub start: NaiveDateTime,
    /// The ending point of the `TimeUsage`.
    pub stop: NaiveDateTime,
    /// An optional description of the `TimeUsage`.
    pub desc: Option<String>,
}

impl Ord for TimeUsage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for TimeUsage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl Eq for TimeUsage {}

impl PartialEq for TimeUsage {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

/// Keeps track of all `TimeUsage`s and their associated categories as well as the the current
/// task being done.
pub struct TimeBook {
    //current_cat: Option<String>,
    //current_cat_start: Option<NaiveDateTime>,
    time_map: HashMap<String, Vec<TimeUsage>>,
}

/// Specifies the time span from which to show records.
#[derive(Clone, Copy)]
pub enum ShownTimeSpan {
    /// Show all records.
    All,
    /// Show records from the past year.
    Year,
    /// Show records from the past month.
    Month,
    /// Show records from the past week.
    Week,
    /// Show records from **only** yesterday.
    Yesterday,
    /// Show records from today.
    Today,
}

impl Default for TimeBook {
    /// Creates a new `TimeBook`.
    fn default() -> Self {
        Self {
            //current_cat: None,
            //current_cat_start: None,
            time_map: HashMap::new(),
        }
    }
}

impl TimeBook {
    /// Adds a new category.
    /// Returns an `Error` if the category already exists.
    pub fn add_category(&mut self, category: String) -> Result<()> {
        // Checking isn't actually necessary, but I consider it to be useful feedback to the user
        if let Entry::Vacant(entry) = self.time_map.entry(category.clone()) {
            entry.insert(Vec::new());
            Ok(())
        } else {
            Err(Error::CategoryExists(category))
        }
    }

    /// Removes a category.
    /// Returns an `Error` if the category doesn't exist.
    pub fn remove_category(&mut self, category: &str) -> Result<()> {
        // Checking isn't actually necessary, but I consider it to be useful feedback to the user
        if self.time_map.contains_key(category) {
            self.time_map.remove(category);
            Ok(())
        } else {
            Err(Error::CategoryDoesntExist(category.to_string()))
        }
    }

    /// Returns all categories.
    pub fn categories(&self) -> Vec<&String> {
        self.time_map.keys().collect()
    }

    /// Creates a new `TimeUsage` and adds it to the `TimeBook` in the specified category.
    /// Returns an `Error` if the category doesn't exist.
    pub fn add_time_usage(
        &mut self,
        category: &str,
        start_time: NaiveDateTime,
        stop_time: NaiveDateTime,
        desc: Option<String>,
    ) -> Result<()> {
        if let Some(usages) = self.time_map.get_mut(category) {
            usages.push(TimeUsage {
                start: start_time,
                stop: stop_time,
                desc,
            });
            usages.sort();
            Ok(())
        } else {
            Err(Error::CategoryDoesntExist(category.to_string()))
        }
    }

    /// Removes time usage from a category.
    /// Returns an `Error` if the category or the time usage with the specified id doesn't exist.
    pub fn remove_time_usage(&mut self, category: &str, id: u64) -> Result<()> {
        if let Some(usages) = self.time_map.get_mut(category) {
            if usages.len() > id as usize {
                usages.remove(id as usize);
                usages.sort();
                Ok(())
            } else {
                Err(Error::TimeUsageDoesntExist(id))
            }
        } else {
            Err(Error::CategoryDoesntExist(category.to_string()))
        }
    }

    /// Returns the time spent on each category from the specifed time span as a `Duration`;
    /// Returns an `Error` if the category doesn't exist.
    pub fn time_spent(&self, category: &str, shown_span: ShownTimeSpan) -> Result<Duration> {
        if let Some(usages) = self.time_map.get(category) {
            let mut total_duration = Duration::zero();

            for usage in usages {
                if TimeBook::in_time_span(usage.start, shown_span) {
                    total_duration = total_duration + (usage.stop - usage.start);
                }
            }
            Ok(total_duration)
        } else {
            Err(Error::CategoryDoesntExist(category.to_string()))
        }
    }

    /// Returns a log of all time usages from the specified time span.
    /// Optionally show logs only from a single category.
    /// Returns an `Error` if the category doesn't exist.
    pub fn time_usage_log(
        &self,
        shown_span: ShownTimeSpan,
        category: Option<&str>,
    ) -> Result<String> {
        if let Some(cat) = category {
            if let Some(usages) = self.time_map.get(cat) {
                let mut st = String::new();

                for usage in usages {
                    st = TimeBook::concat_usage(st, usage, shown_span, cat);
                }

                Ok(st)
            } else {
                Err(Error::CategoryDoesntExist(cat.to_string()))
            }
        } else {
            // The following bit of code is quite messy.
            // We build a chronologically ordered log string that contains entries from EVERY
            // category.
            // We use the fact that every TimeUsage list is always sorted.

            // Calculate the total number of TimeUsage entries in all categories.
            // Also create a new hash map containing indicies to every category.
            let mut index_sum = 0;
            let mut index_map = HashMap::new();
            for (category, val) in &self.time_map {
                index_sum += val.len();
                index_map.insert(category, 0usize);
            }

            // The indicies start at 0 but every time a category is accessed the mapped index is
            // increased by one.

            // Basically loop through every element in all the categories' TimeUsage lists.
            // Every iteration look through all categories for the TimeUsage that is the oldest.
            // Append that usage to the log.
            // Increment that category's index by one because we don't want for it to get re-added.

            let mut log = String::new();

            for _ in 0..index_sum {
                // Random category as the oldest for a starting point
                let mut oldest = index_map.keys().next().unwrap().to_owned();

                for (cat, cat_index) in &index_map {
                    // Both of these exist so unwrap is ok.
                    let cat_items = self.time_map.get(cat.to_owned()).unwrap();
                    let oldest_items = self.time_map.get(oldest).unwrap();

                    let cat_index = cat_index.to_owned();
                    let oldest_index = index_map[oldest];

                    // If the index is out of bounds, every TimeUsage has been already added from this
                    // category.
                    if cat_items.len() <= cat_index {
                        continue;
                    }

                    // If the index is out of bounds, every TimeUsage has been already added from the
                    // oldest category.
                    if oldest_items.len() <= oldest_index {
                        oldest = cat;
                        continue;
                    }

                    // Check if the oldest item should change
                    if cat_items[cat_index] < oldest_items[oldest_index] {
                        oldest = cat;
                    }
                }

                let oldest_index = index_map[oldest];
                let oldest_usage = self.time_map[oldest].get(oldest_index).unwrap();

                // Increment the index map for the oldest category
                index_map.insert(oldest, index_map[oldest] + 1);

                log = TimeBook::concat_usage(log, oldest_usage, shown_span, oldest);
            }

            Ok(log)
        }
    }

    fn concat_usage(
        mut s: String,
        usage: &TimeUsage,
        shown_span: ShownTimeSpan,
        cat: &str,
    ) -> String {
        if TimeBook::in_time_span(usage.start, shown_span) {
            let fstring = "%-d/%-m/%Y %H:%M";
            let mut elem = format!(
                "{} - {}: {}",
                usage.start.format(fstring),
                usage.stop.format(fstring),
                cat
            );
            if let Some(d) = &usage.desc {
                elem = format!("{}\n\t{}", elem, d);
            }
            s = format!("{}\n\n{}", elem, s);
        }
        s
    }

    fn in_time_span(start_time: NaiveDateTime, span: ShownTimeSpan) -> bool {
        let now = Local::now().naive_local();
        let today = now.date();

        match span {
            ShownTimeSpan::All => true,
            ShownTimeSpan::Year => today - start_time.date() <= Duration::days(365),
            ShownTimeSpan::Month => today - start_time.date() <= Duration::weeks(4),
            ShownTimeSpan::Week => today - start_time.date() <= Duration::weeks(1),
            ShownTimeSpan::Yesterday => today.pred() == start_time.date(),
            ShownTimeSpan::Today => today == start_time.date(),
        }
    }
}

// Due to the small nature of this project I have only written integration test style large
// tests that don't follow the AAA pattern. Basically I have merged tests together.
//
// Splitting the tests into smaller units would be more work and wouldn't provide much practical
// benefit.

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn category_can_be_added_and_removed() {
        let mut book = TimeBook::default();

        book.add_category("test".to_string()).unwrap();

        assert!(book.categories().contains(&&("test".to_string())));

        book.add_category("test1".to_string()).unwrap();

        assert_eq!(book.categories().len(), 2);

        book.remove_category("test").unwrap();

        assert!(book.categories().contains(&&("test1".to_string())));
        assert_eq!(book.categories().len(), 1);
    }

    #[test]
    fn trying_to_remove_nonexistant_category_fails() {
        let mut book = TimeBook::default();
        assert_eq!(
            book.remove_category("test").unwrap_err(),
            Error::CategoryDoesntExist("test".to_string())
        );
    }

    #[test]
    fn adding_already_existing_category_fails() {
        let mut book = TimeBook::default();
        book.add_category("test".to_string()).unwrap();
        assert_eq!(
            book.add_category("test".to_string()).unwrap_err(),
            Error::CategoryExists("test".to_string())
        );
    }

    #[test]
    fn time_usage_cannot_be_added_to_category_that_doesnt_exist() {
        let mut book = TimeBook::default();
        assert_eq!(
            book.add_time_usage(
                "test",
                NaiveDate::from_ymd(2022, 1, 1).and_hms(9, 0, 0),
                NaiveDate::from_ymd(2022, 1, 1).and_hms(10, 0, 0),
                None
            )
            .unwrap_err(),
            Error::CategoryDoesntExist("test".to_string())
        );
    }

    #[test]
    fn time_usage_can_be_added_and_removed_from_multiple_categories() {
        let mut book = TimeBook::default();

        book.add_category("test".to_string()).unwrap();
        book.add_category("test_second".to_string()).unwrap();

        book.add_time_usage(
            "test",
            NaiveDate::from_ymd(2022, 1, 1).and_hms(9, 0, 0),
            NaiveDate::from_ymd(2022, 1, 1).and_hms(10, 0, 0),
            None,
        )
        .unwrap();

        assert_eq!(
            book.time_spent("test", ShownTimeSpan::All).unwrap(),
            Duration::hours(1)
        );
        assert_eq!(
            book.time_spent("test_second", ShownTimeSpan::All).unwrap(),
            Duration::zero()
        );

        book.add_time_usage(
            "test_second",
            NaiveDate::from_ymd(2022, 1, 1).and_hms(9, 0, 0),
            NaiveDate::from_ymd(2022, 1, 1).and_hms(10, 0, 0),
            None,
        )
        .unwrap();

        assert_eq!(
            book.time_spent("test", ShownTimeSpan::All).unwrap(),
            Duration::hours(1)
        );
        assert_eq!(
            book.time_spent("test_second", ShownTimeSpan::All).unwrap(),
            Duration::hours(1)
        );

        book.remove_time_usage("test", 0).unwrap();

        assert_eq!(
            book.time_spent("test", ShownTimeSpan::All).unwrap(),
            Duration::zero()
        );
        assert_eq!(
            book.time_spent("test_second", ShownTimeSpan::All).unwrap(),
            Duration::hours(1)
        );
    }

    fn time_book_with_usages() -> TimeBook {
        let now = Local::now().naive_local();
        let yesterday = Local::now().naive_local() - Duration::days(1);
        let week_ago = Local::now().naive_local() - Duration::weeks(1);
        let month_ago = Local::now().naive_local() - Duration::weeks(4);
        let year_ago = Local::now().naive_local() - Duration::days(365);
        let two_years_ago = Local::now().naive_local() - Duration::days(700);

        let mut book = TimeBook::default();

        book.add_category("test".to_string()).unwrap();

        book.add_time_usage(
            "test",
            now,
            now + Duration::minutes(30),
            Some("Time usage of today".to_string()),
        )
        .unwrap();

        book.add_time_usage("test", yesterday, yesterday + Duration::minutes(30), None)
            .unwrap();

        book.add_time_usage(
            "test",
            week_ago,
            week_ago + Duration::minutes(30),
            Some("Week ago".to_string()),
        )
        .unwrap();

        book.add_time_usage("test", month_ago, month_ago + Duration::minutes(30), None)
            .unwrap();

        book.add_time_usage(
            "test",
            year_ago,
            year_ago + Duration::minutes(30),
            Some("A Year ago".to_string()),
        )
        .unwrap();

        book.add_time_usage(
            "test",
            two_years_ago,
            two_years_ago + Duration::minutes(30),
            None,
        )
        .unwrap();

        book
    }

    #[test]
    fn time_usage_is_reported_correctly_for_the_correct_shown_time_spans() {
        let book = time_book_with_usages();

        assert_eq!(
            book.time_spent("test", ShownTimeSpan::Today).unwrap(),
            Duration::minutes(30)
        );
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::Yesterday).unwrap(),
            Duration::minutes(30)
        );
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::Week).unwrap(),
            Duration::minutes(90)
        );
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::Month).unwrap(),
            Duration::minutes(120)
        );
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::Year).unwrap(),
            Duration::minutes(150)
        );
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::All).unwrap(),
            Duration::minutes(180)
        );
    }

    #[test]
    fn time_spent_returns_err_for_nonexistant_category() {
        let book = TimeBook::default();
        assert_eq!(
            book.time_spent("test", ShownTimeSpan::All).unwrap_err(),
            Error::CategoryDoesntExist("test".to_string())
        );
    }

    #[test]
    fn time_usage_log_is_written_correctly_for_the_correct_shown_time_spans() {
        let book = time_book_with_usages();

        let now = Local::now().naive_local();
        let yesterday = Local::now().naive_local() - Duration::days(1);
        let week_ago = Local::now().naive_local() - Duration::weeks(1);
        let month_ago = Local::now().naive_local() - Duration::weeks(4);
        let year_ago = Local::now().naive_local() - Duration::days(365);
        let two_years_ago = Local::now().naive_local() - Duration::days(700);

        let fstring = "%-d/%-m/%Y %H:%M";

        let now_str = format!(
            "{} - {}: test\n\tTime usage of today\n\n",
            now.format(fstring),
            (now + Duration::minutes(30)).format(fstring)
        );

        let yesterday_str = format!(
            "{} - {}: test\n\n",
            yesterday.format(fstring),
            (yesterday + Duration::minutes(30)).format(fstring)
        );

        let week_str = format!(
            "{} - {}: test\n\tWeek ago\n\n",
            week_ago.format(fstring),
            (week_ago + Duration::minutes(30)).format(fstring)
        );

        let month_str = format!(
            "{} - {}: test\n\n",
            month_ago.format(fstring),
            (month_ago + Duration::minutes(30)).format(fstring)
        );

        let year_str = format!(
            "{} - {}: test\n\tA Year ago\n\n",
            year_ago.format(fstring),
            (year_ago + Duration::minutes(30)).format(fstring)
        );

        let two_years_ago_str = format!(
            "{} - {}: test\n\n",
            two_years_ago.format(fstring),
            (two_years_ago + Duration::minutes(30)).format(fstring)
        );

        assert_eq!(
            book.time_usage_log(ShownTimeSpan::Today, None).unwrap(),
            now_str
        );
        assert_eq!(
            book.time_usage_log(ShownTimeSpan::Yesterday, None).unwrap(),
            yesterday_str
        );
        assert_eq!(
            book.time_usage_log(ShownTimeSpan::Week, None).unwrap(),
            now_str.clone() + &yesterday_str + &week_str
        );
        assert_eq!(
            book.time_usage_log(ShownTimeSpan::All, None).unwrap(),
            now_str + &yesterday_str + &week_str + &month_str + &year_str + &two_years_ago_str
        );
    }

    #[test]
    fn time_usage_log_is_written_correctly_for_different_category_specifications() {
        let mut book = TimeBook::default();

        book.add_category("test".to_string()).unwrap();
        book.add_category("test_second".to_string()).unwrap();

        book.add_time_usage(
            "test_second",
            NaiveDate::from_ymd(2022, 1, 1).and_hms(9, 0, 0),
            NaiveDate::from_ymd(2022, 1, 1).and_hms(10, 0, 0),
            None,
        )
        .unwrap();

        book.add_time_usage(
            "test",
            NaiveDate::from_ymd(2022, 1, 1).and_hms(10, 0, 0),
            NaiveDate::from_ymd(2022, 1, 1).and_hms(11, 0, 0),
            None,
        )
        .unwrap();

        assert_eq!(
            book.time_usage_log(ShownTimeSpan::All, None).unwrap(),
            "1/1/2022 10:00 - 1/1/2022 11:00: test\n\n1/1/2022 09:00 - 1/1/2022 10:00: test_second\n\n"
        );

        assert_eq!(
            book.time_usage_log(ShownTimeSpan::All, Some("test"))
                .unwrap(),
            "1/1/2022 10:00 - 1/1/2022 11:00: test\n\n"
        );

        assert_eq!(
            book.time_usage_log(ShownTimeSpan::All, Some("test_second"))
                .unwrap(),
            "1/1/2022 09:00 - 1/1/2022 10:00: test_second\n\n"
        );
    }

    #[test]
    fn time_usage_log_returns_err_for_nonexistant_category() {
        let book = TimeBook::default();
        assert_eq!(
            book.time_usage_log(ShownTimeSpan::All, Some("test"))
                .unwrap_err(),
            Error::CategoryDoesntExist("test".to_string())
        );
    }
}