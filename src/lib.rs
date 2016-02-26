#![cfg_attr(feature = "dev", allow(ptr_arg))]
#![deny(missing_docs,missing_debug_implementations)]
#![deny(missing_copy_implementations,trivial_casts)]
#![deny(trivial_numeric_casts, unsafe_code, unstable_features)]
#![deny(unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

//! A simple wrapper of keen.io's api v3
extern crate chrono;
extern crate itertools;
extern crate hyper;

use itertools::Itertools;
use std::*;
use chrono::*;
use hyper::Client;

/// A Keen.io client
#[derive(Debug,Clone)]
pub struct KeenClient {
    key: String,
    project: String,
    timeout: Option<time::Duration>,
}

impl KeenClient {
    /// Create a new keen.io client with read key and a project's hash.
    pub fn new(key: &str, project: &str) -> KeenClient {
        KeenClient {
            key: key.into(),
            project: project.into(),
            timeout: None,
        }
    }

    /// Set the client timeout.
    pub fn timeout(&mut self, time: time::Duration) {
        self.timeout = Some(time)
    }

    /// Generate a new query with current client.
    pub fn query(&self, m: Metric, c: String, timeframe: TimeFrame) -> KeenQuery {
        KeenQuery {
            client: self,
            metric: m,
            collection: c,
            timeframe: timeframe,
            group_by: vec![],
            filters: vec![],
            interval: None,
            max_age: None,
            others: vec![]
        }
    }
}

/// Type represents a keen io query
#[derive(Debug,Clone)]
pub struct KeenQuery<'a> {
    client: &'a KeenClient,
    metric: Metric,
    collection: String,
    timeframe: TimeFrame,
    group_by: Vec<String>,
    filters: Vec<Filter>,
    interval: Option<Interval>,
    others: Vec<(String, String)>,
    max_age: Option<usize>,
}

impl<'a> KeenQuery<'a> {
    /// Add a new group by condition to current query.
    pub fn group_by(&mut self, g: &str) -> &mut KeenQuery<'a> {
        self.group_by.push(g.into());
        self
    }

    /// Add a new filter to current query.
    pub fn filter(&mut self, f: Filter) -> &mut KeenQuery<'a> {
        self.filters.push(f);
        self
    }

    /// Add an interval to current query.
    pub fn interval(&mut self, i: Interval) -> &mut KeenQuery<'a> {
        self.interval = Some(i);
        self
    }

    /// Use cache with and set cache timeout to current query.
    pub fn max_age(&mut self, age: usize) -> &mut KeenQuery<'a> {
        self.max_age = Some(age);
        self
    }

    /// Other customized parameters that sent to keen io
    pub fn other(&mut self, key: &str, value: &str) -> &mut KeenQuery<'a> {
        self.others.push((key.into(), value.into()));
        self
    }

    /// Generate the query url.
    pub fn url(&self) -> String {
        let mut s = format!("https://api.keen.io/3.\
                             0/projects/{project}/queries/{metric}api_key={key}&event_collection=\
                             {collection}&group_by={group}&timezone=UTC&timeframe={timeframe}&fil\
                             ters={filters}",
                            project = self.client.project,
                            metric = self.metric,
                            key = self.client.key,
                            collection = self.collection,
                            group = KeenQuery::format_group(&self.group_by),
                            timeframe = self.timeframe,
                            filters = KeenQuery::format_filter(&self.filters));
        self.interval.as_ref().map(|i| s.push_str(&format!("&interval={}", i)));
        self.max_age.as_ref().map(|a| s.push_str(&format!("&max_age={}", a)));

        for &(ref k, ref v) in self.others.iter() {
            s.push_str(&format!("&{}={}", k, v));
        }
        s
    }

    fn format_group(g: &[String]) -> String {
        let mut s = String::new();
        s.push('[');
        s.push_str(&g.iter()
                     .map(|s| {
                         let mut r = r#"""#.to_owned();
                         r.push_str(&s);
                         r.push('"');
                         r
                     })
                     .join(","));
        s.push(']');
        s
    }

    fn format_filter(f: &[Filter]) -> String {
        let mut s = String::new();
        s.push('[');
        s.push_str(&f.iter()
                     .map(|s| format!("{}", s))
                     .join(","));
        s.push(']');
        s
    }

    /// Get the query data. The result is a hyper::Result<hyper::client::Response>
    pub fn data(&self) -> hyper::Result<hyper::client::Response> {
        self.client
            .timeout
            .map(|t| {
                let mut client = Client::new();
                client.set_read_timeout(Some(t));
                client.set_write_timeout(Some(t));
                client
            })
            .unwrap_or(Client::new())
            .get(&self.url())
            .send()
    }
}

/// Type to define a timeframe.
/// There are two variants:
#[derive(Debug,Clone)]
pub enum TimeFrame {
    ///  1. relative time from now.
    Relative(String),
    ///  2. absolute timeframe with which you should specify a start time and a end time.
    Absolute(DateTime<UTC>, DateTime<UTC>),
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TimeFrame::*;
        let s = match *self {
            Relative(ref s) => s.clone(),
            Absolute(ref f, ref t) => {
                format!(r#"{{"start":"{}","end":"{}"}}"#, f.to_rfc3339(), t.to_rfc3339())
            }
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Filter::*;
        let (name, op, value) = match *self {
            Eq(ref f, ref v) => (f, "eq", v),
            Ne(ref f, ref v) => (f, "ne", v),
            Lt(ref f, ref v) => (f, "lt", v),
            Gt(ref f, ref v) => (f, "gt", v),
            Lte(ref f, ref v) => (f, "lte", v),
            Gte(ref f, ref v) => (f, "gte", v),
            NotContains(ref f, ref v) => (f, "not_contains", v),
            Contains(ref f, ref v) => (f, "contains", v),
            Exists(ref f, ref v) => (f, "exists", v),
            In(ref f, ref v) => (f, "in", v)
        };
        write!(f,
               r#"{{"property_name":"{}","property_value":{},"operator":"{}"}}"#,
               name,
               value,
               op)
    }
}

/// Type define a filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// this means a == b
    Eq(String, String),
    /// this means a != b
    Ne(String, String),
    /// this means a < b
    Lt(String, String),
    /// this means a > b
    Gt(String, String),
    /// this means a <= b
    Lte(String, String),
    /// this means a >= b
    Gte(String, String),
    /// this means a ∉ b
    NotContains(String, String),
    /// this means a ∈ b
    Contains(String, String),
    /// this means ∃a ∈ b
    Exists(String, String),
    /// this means in
    In(String, String)
}

impl Filter {
    /// Generate a new eq operator
    pub fn eq<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Eq(name.into(), value.to_filter())
    }
    /// Generate a new ne operator
    pub fn ne<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Ne(name.into(), value.to_filter())
    }
    /// Generate a new lt operator
    pub fn lt<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Lt(name.into(), value.to_filter())
    }
    /// Generate a new gt operator
    pub fn gt<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Gt(name.into(), value.to_filter())
    }
    /// Generate a new lte operator
    pub fn lte<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Lte(name.into(), value.to_filter())
    }
    /// Generate a new gte operator
    pub fn gte<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Gte(name.into(), value.to_filter())
    }
    /// Generate a new contains operator
    pub fn contains<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Contains(name.into(), value.to_filter())
    }
    /// Generate a new not_contains operator
    pub fn not_contains<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::NotContains(name.into(), value.to_filter())
    }
    /// Generate a new exists operator
    pub fn exists<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::Exists(name.into(), value.to_filter())
    }

    /// Generate a new in operator
    pub fn isin<T>(name: &str, value: T) -> Filter
        where T: ToFilterValue
    {
        Filter::In(name.into(), value.to_filter())
    }
}

/// Trait represents a type that can be used as a filter's value.
#[allow(missing_docs)]
pub trait ToFilterValue {
    fn to_filter(&self) -> String;
}

macro_rules! numeric_impl {
    ($($t: ty)+) => {
        $(impl ToFilterValue for $t {
            fn to_filter(&self) -> String {
                format!("{}", self)
            }
        })+
    }
}

numeric_impl!(i32 i64 isize usize u32 u64 f32 f64);

impl<'a> ToFilterValue for &'a str {
    fn to_filter(&self) -> String {
        format!(r#""{}""#, self)
    }
}

impl ToFilterValue for String {
    fn to_filter(&self) -> String {
        format!(r#""{}""#, self)
    }
}

impl<'a> ToFilterValue for &'a String {
    fn to_filter(&self) -> String {
        format!(r#""{}""#, self)
    }
}

impl<I: ToFilterValue> ToFilterValue for Vec<I> {
    fn to_filter(&self) -> String {
        let s = self.iter().map(|i| i.to_filter()).join(",");
        format!("[{}]", s)
    }
}
/// Type defines a metric to query.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Metric {
    Sum(String),
    Count,
    CountUnique(String),
    Minimum(String),
    Maximum(String),
    Average(String),
    SelectUnique(String),
    Extraction,
    Percentile(String, f64),
    Median(String),
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Metric::*;
        match *self {
            Sum(ref s) => write!(f, r#"sum?target_property={}&"#, s),
            Count => write!(f, r#"count?"#),
            CountUnique(ref s) => write!(f, r#"count_unique?target_property={}&"#, s),
            Minimum(ref s) => write!(f, r#"minimum?target_property={}&"#, s),
            Maximum(ref s) => write!(f, r#"maximum?target_property={}&"#, s),
            Average(ref s) => write!(f, r#"average?target_property={}&"#, s),
            SelectUnique(ref s) => write!(f, r#"select_unique?target_property={}&"#, s),
            Extraction => write!(f, r#"extraction"#),
            Percentile(ref s, p) =>
                write!(f, r#"percentile?target_property={}&percentile={}&"#, s, p),
            Median(ref s) => write!(f, r#"median?target_property={}&"#, s),
        }
    }
}

/// Type defines interval.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
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
    let cl = KeenClient::new("your keen io api key", "your project id");
    let m = Metric::CountUnique("metric1".into());
    let c = "collection_name".into();
    let from = UTC::now() - Duration::days(2);
    let from_str = format!("{}", from.to_rfc3339());
    let to = UTC::now();
    let to_str = format!("{}", to.to_rfc3339());
    let t = TimeFrame::Absolute(from, to);
    let mut q = cl.query(m, c, t);
    q.group_by("group1")
     .group_by("group2")
     .filter(Filter::gt("id", 458888))
     .filter(Filter::lte("id", 460000))
     .interval(Interval::Monthly);
    q.other("uuid", "12345678-1234-1234-1234-123456789101");
    assert_eq!(q.url(), format!(r#"https://api.keen.io/3.0/projects/your project id/queries/count_unique?target_property=metric1&api_key=your keen io api key&event_collection=collection_name&group_by=["group1","group2"]&timezone=UTC&timeframe={{"start":"{}","end":"{}"}}&filters=[{{"property_name":"id","property_value":458888,"operator":"gt"}},{{"property_name":"id","property_value":460000,"operator":"lte"}}]&interval=monthly&uuid=12345678-1234-1234-1234-123456789101"#, from_str, to_str));
}
