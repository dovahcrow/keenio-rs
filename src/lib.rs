extern crate chrono;
extern crate itertools;
extern crate hyper;

use itertools::Itertools;
use std::*;
use chrono::*;
use hyper::Client;

pub struct KeenClient {
    key: String,
    project: String
}

impl KeenClient {
    pub fn new(key: &str, project: &str) -> KeenClient {
        KeenClient {
            key: key.into(),
            project: project.into()
        }
    }
    pub fn query(&self, m: Metric, c: String, timeframe: TimeFrame) -> KeenQuery {
        KeenQuery {
            client: self,
            debug: false,
            metric: m,
            collection: c,
            timeframe: timeframe,
            group_by: vec![],
            filters: vec![],
            interval: None
        }
    }
}

pub struct KeenQuery<'a> {
    client: &'a KeenClient,
    debug: bool,
    metric: Metric,
    collection: String,
    timeframe: TimeFrame,
    group_by: Vec<String>,
    filters: Vec<Filter>,
    interval: Option<Interval>
}

impl<'a> KeenQuery<'a> {
    pub fn debug(&mut self, d: bool) -> &mut KeenQuery<'a> {
        self.debug = d;
        self
    }
    pub fn add_group(&mut self, g: &str) -> &mut KeenQuery<'a> {
        self.group_by.push(g.into());
        self
    }
    pub fn add_filter(&mut self, f: Filter) -> &mut KeenQuery<'a> {
        self.filters.push(f);
        self
    }
    pub fn interval(&mut self, i: Interval) -> &mut KeenQuery<'a> {
        self.interval = Some(i);
        self
    }
    pub fn url(&self) -> String {
        let mut s = format!(
            "https://api.keen.io/3.0/projects/{project}/queries/{metric}&api_key={key}&event_collection={collection}&group_by={group}&timezone=UTC&timeframe={timeframe}&filters={filters}",
            project = self.client.project,
            metric = self.metric,
            key = self.client.key,
            collection = self.collection,
            group = KeenQuery::format_group(&self.group_by),
            timeframe = self.timeframe,
            filters = KeenQuery::format_filter(&self.filters));
        self.interval.as_ref().map(|i| s.push_str(&format!("&interval={}", i)));
        s
    }
    fn format_group(g: &[String]) -> String {
        let mut s = String::new();
        s.push('[');
        s.push_str(&g.iter().map(|s| {
            let mut r = r#"""#.to_owned();
            r.push_str(&s);
            r.push('"');
            r
        }).fold1(|mut a, s| {a.push(',');a.push_str(&s);a}).unwrap_or("".into()));
        s.push(']');
        s
    }
    fn format_filter(f: &[Filter]) -> String {
        let mut s = String::new();
        s.push('[');
        s.push_str(&f.iter().map(|s| {
            format!("{}", s)
        }).fold1(|mut a, s| {a.push(',');a.push_str(&s);a}).unwrap_or("".into()));
        s.push(']');
        s
    }
    pub fn data(&self) -> hyper::Result<hyper::client::Response> {
        Client::new().get(&self.url()).send()
    }
}

pub enum TimeFrame {
    Rel(String),
    Abs(DateTime<UTC>, DateTime<UTC>)
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TimeFrame::*;
        let s = match self {
            &Rel(ref s) => s.clone(),
            &Abs(f, t) => {
                format!(r#"{{"start":"{}","end":"{}"}}"#, f, t)
            }
        };
        write!(f, "{}", s)
    }
}

pub struct Filter {
    property_name: String,
    property_value: String,
    operator: Operator
}

impl Filter {
    pub fn new(name: &str, operator: Operator, value: &str) -> Filter {
        Filter {
            property_name: name.into(),
            property_value: value.into(),
            operator: operator
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"{{"property_name":"{}","property_value":"{}","operator":"{}"}}"#, self.property_name, self.property_value, self.operator)
    }
}

pub enum Operator {
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operator::*;
        let s = match self {
            &Eq => "eq",
            &Ne => "ne",
            &Lt => "lt",
            &Gt => "gt",
            &Lte => "lte",
            &Gte => "gte"
        };
        write!(f, "{}", s)
    }
}

pub enum Metric {
    Count,
    CountUnique(String)
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Metric::*;
        match self {
            &Count => write!(f, r#"count?"#),
            &CountUnique(ref s) => write!(f, r#"count_unique?target_property={}"#, s),
        }
    }
}

#[derive(Debug)]
pub enum Interval {
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[test]
fn it_works() {
}
