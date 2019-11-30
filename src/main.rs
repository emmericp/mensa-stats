use std::fs::File;
use std::io;

use chrono::{NaiveDate, Datelike};
use clap::App;
use csv::Writer;
use serde::Deserialize;
use tzdata::Timezone;

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
