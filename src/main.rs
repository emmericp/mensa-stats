use std::fs::File;
use std::io;

use chrono::{NaiveDate, Datelike};
use clap::App;
use csv::Writer;
use serde::Deserialize;
use tzdata::Timezone;
use itertools::Itertools;
use float_ord::FloatOrd;

#[derive(Deserialize, Debug)]
struct StatsEntry {
    target: String,
    datapoints: Vec<(f64, i64)>,
}

fn main() {
    let args = App::new("MensaStats")
        .arg_from_usage("--ap-stats <FILE> json dump of LRZ AP stats")
        .arg_from_usage("--mode <MODE> single, average, or overview")
        .arg_from_usage("--day <DAY> day for single mode (YYYY-MM-DD)")
        .get_matches();
    let raw: Vec<StatsEntry> = serde_json::from_reader(File::open(args.value_of("ap-stats").unwrap()).unwrap()).unwrap();
    assert_eq!(raw.len(), 1);
    let data = &raw.get(0).unwrap().datapoints;
    match args.value_of("mode") {
        Some("single") => {
            single_day(data, NaiveDate::parse_from_str(args.value_of("day").unwrap(), "%Y-%m-%d").ok().unwrap())
        }
        Some("overview") => {
            overview(data)
        }
        _ => {
            panic!()
        }
    }
}

fn single_day(stats: &Vec<(f64, i64)>, day: NaiveDate) {
    let timezone = Timezone::new("Europe/Berlin").unwrap();
    let mut csv = Writer::from_writer(io::stdout());
    csv.serialize(("timestamp", "value")).ok();
    stats.into_iter().map(|(value, time)| {
        (value, timezone.unix(*time, 0))
    }).filter(|(_, time)| {
        let (y, m, d) = time.date();
        y == day.year() && m == day.month() as i32 && d == day.day() as i32
    }).for_each(|(value, time)| {
        csv.serialize((time.format("%H:%M"), *value)).ok();
    });
}

fn overview(stats: &Vec<(f64, i64)>) {
    let timezone = Timezone::new("Europe/Berlin").unwrap();
    let mut csv = Writer::from_writer(io::stdout());
    csv.serialize(("day", "peak")).ok();
    // yes, a simple graphite query could also get this result directly
    let days = stats.into_iter().map(|(value, time)| {
        (value, timezone.unix(*time, 0))
    }).group_by(|(_, time)| {
        time.format("%Y-%m-%d")
    });
    for (day, group) in &days {
        if let Some((max, _)) = group.max_by_key(|(value, _)| { FloatOrd(**value) }) {
            csv.serialize((day, max)).ok();
        } else {
            csv.serialize((day, -1));
        }
    }
}
