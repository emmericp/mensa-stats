use std::fs::File;
use std::io;

use chrono::{NaiveDate, Datelike, NaiveTime};
use clap::App;
use csv::Writer;
use serde::Deserialize;
use tzdata::{Timezone, Datetime};
use itertools::Itertools;
use float_ord::FloatOrd;
use stats::median;
use std::collections::{HashMap, BTreeMap};
use std::io::Stdout;

#[derive(Deserialize, Debug)]
struct StatsEntry {
    target: String,
    datapoints: Vec<(f64, i64)>,
}

fn main() {
    let args = App::new("MensaStats")
        .arg_from_usage("--ap-stats <FILE> 'json dump of LRZ AP stats'")
        .arg_from_usage("--mode <MODE> 'single, average, or overview'")
        .arg_from_usage("--day [DAY] 'day for single mode (YYYY-MM-DD)'")
        .arg_from_usage("--since [START_DAY] start day for average mode (YYYY-MM-DD)")
        .arg_from_usage("--until [END_DAY] 'end day for average mode (YYYY-MM-DD)'")
        .get_matches();
    let raw: Vec<StatsEntry> = serde_json::from_reader(File::open(args.value_of("ap-stats").unwrap()).unwrap()).unwrap();
    assert_eq!(raw.len(), 1);
    let data = &raw.get(0).unwrap().datapoints;
    match args.value_of("mode") {
        Some("single") => {
            single_day(data, NaiveDate::parse_from_str(args.value_of("day").unwrap(), "%Y-%m-%d").ok().unwrap())
        }
        Some("average") => {
            average(
                data,
                NaiveDate::parse_from_str(args.value_of("since").unwrap(), "%Y-%m-%d").ok().unwrap(),
                NaiveDate::parse_from_str(args.value_of("until").unwrap(), "%Y-%m-%d").ok().unwrap(),
            )
        }
        Some("overview") => {
            overview(data)
        }
        _ => {
            panic!("invalid mode")
        }
    }
}

fn single_day(stats: &Vec<(f64, i64)>, day: NaiveDate) {
    let timezone = Timezone::new("Europe/Berlin").unwrap();
    let mut csv = Writer::from_writer(io::stdout());
    csv.serialize(("timestamp", "value")).ok();
    stats.into_iter().map(|(value, time)| {
        (*value, timezone.unix(*time, 0))
    }).filter(|(_, time)| {
        let (y, m, d) = time.date();
        y == day.year() && m == day.month() as i32 && d == day.day() as i32
    }).for_each(|(value, time)| {
        csv.serialize((time.format("%H:%M"), value)).ok();
    });
}

fn get_stats_for_weekday(stats: &Vec<(f64, Datetime)>) -> Vec<((i32, i32, i32, i32), f64)> {
    // detect holidays: peak is less than 10% the median peak for that day
    let by_day = stats.into_iter().map(|(value, time)| {
        (time.date(), (*value, time))
    }).into_group_map();
    let max_by_day: HashMap<(i32, i32, i32), f64> = by_day.iter().map(|(day, vec)| {
        (*day, vec.iter().max_by_key(|(value, _)| { FloatOrd(*value) }).unwrap().0)
    }).collect();
    let median = median(max_by_day.values().map(|val| *val)).unwrap();
    let mut num_days = 0;
    let mut average: BTreeMap<(i32, i32, i32, i32), f64> = BTreeMap::new();
    by_day.iter().filter(|(day, _)| {
        *max_by_day.get(day).unwrap() >= median * 0.1
    }).for_each(|(_, vec)| {
        num_days += 1;
        vec.iter().for_each(|(value, time)| {
            *average.entry(time.time()).or_insert(0.0) += value;
        })
    });
    return average.into_iter().map(|(date, val)| {
        (date, val / (num_days as f64))
    }).collect();
}

fn print_stats_for_weekday(csv: &mut Writer<Stdout>, day: &str, by_weekday: &HashMap<String, Vec<(f64, Datetime)>>) {
    for (time, value) in get_stats_for_weekday(by_weekday.get(day).unwrap()) {
        csv.serialize((&day, NaiveTime::from_hms(time.0 as u32, time.1 as u32, time.2 as u32).format("%H:%M").to_string(), value)).ok();
    }
}

fn average(stats: &Vec<(f64, i64)>, start: NaiveDate, end: NaiveDate) {
    let timezone = Timezone::new("Europe/Berlin").unwrap();
    let mut csv = Writer::from_writer(io::stdout());
    csv.serialize(("weekday", "timestamp", "value")).ok();
    let by_weekday = stats.into_iter().map(|(value, time)| {
        (*value, timezone.unix(*time, 0))
    }).filter(|(_, time)| {
        let (y, m, d) = time.date();
        let date = NaiveDate::from_ymd(y, m as u32, d as u32);
        date >= start && date <= end
    }).map(|(value, time)| {
        (time.format("%A"), (value, time))
    }).into_group_map();
    print_stats_for_weekday(&mut csv, "Monday", &by_weekday);
    print_stats_for_weekday(&mut csv, "Tuesday", &by_weekday);
    print_stats_for_weekday(&mut csv, "Wednesday", &by_weekday);
    print_stats_for_weekday(&mut csv, "Thursday", &by_weekday);
    print_stats_for_weekday(&mut csv, "Friday", &by_weekday);
}

fn overview(stats: &Vec<(f64, i64)>) {
    let timezone = Timezone::new("Europe/Berlin").unwrap();
    let mut csv = Writer::from_writer(io::stdout());
    csv.serialize(("day", "peak")).ok();
    // yes, a simple graphite query could also get this result directly
    let days = stats.into_iter().map(|(value, time)| {
        (*value, timezone.unix(*time, 0))
    }).group_by(|(_, time)| {
        time.format("%Y-%m-%d")
    });
    for (day, group) in &days {
        if let Some((max, _)) = group.max_by_key(|(value, _)| { FloatOrd(*value) }) {
            csv.serialize((day, max)).ok();
        } else {
            csv.serialize((day, -1)).ok();
        }
    }
}
