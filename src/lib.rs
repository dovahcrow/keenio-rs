extern crate chrono;
extern crate itertools;
extern crate hyper;

use itertools::Itertools;
use std::*;
use chrono::*;
use hyper::Client;

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum TimeFrame {
    Relative(String),
    Absolute(DateTime<UTC>, DateTime<UTC>)
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TimeFrame::*;
        let s = match self {
            &Relative(ref s) => s.clone(),
            &Absolute(f, t) => {
                format!(r#"{{"start":"{}","end":"{}"}}"#, f, t)
            }
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
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

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone)]
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
    Median(String)
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Metric::*;
        match self {
            &Sum(ref s) => write!(f, r#"sum?target_property={}"#, s),
            &Count => write!(f, r#"count?"#),
            &CountUnique(ref s) => write!(f, r#"count_unique?target_property={}"#, s),
            &Minimum(ref s) => write!(f, r#"minimum?target_property={}"#, s),
            &Maximum(ref s) => write!(f, r#"maximum?target_property={}"#, s),
            &Average(ref s) => write!(f, r#"average?target_property={}"#, s),
            &SelectUnique(ref s) => write!(f, r#"select_unique?target_property={}"#, s),
            &Extraction => write!(f, r#"extraction"#),
            &Percentile(ref s, p) => write!(f, r#"percentile?target_property={}&percentile={}"#, s, p),
            &Median(ref s) => write!(f, r#"median?target_property={}"#, s),
        }
    }
}

#[derive(Debug, Clone)]
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
    let from_str = format!("{}", from);
    let to =  UTC::now();
    let to_str = format!("{}", to);
    let t = TimeFrame::Absolute(from, to);
    let mut q = cl.query(m, c, t);
    q.add_group("group1");
    q.add_group("group2");
    q.add_filter(Filter::new("id", Operator::Gt, "458888"));
    q.add_filter(Filter::new("id", Operator::Lte, "460000"));
    q.interval(Interval::Monthly);
    assert_eq!(q.url(), format!(r#"https://api.keen.io/3.0/projects/your project id/queries/count_unique?target_property=metric1&api_key=your keen io api key&event_collection=collection_name&group_by=["group1","group2"]&timezone=UTC&timeframe={{"start":"{}","end":"{}"}}&filters=[{{"property_name":"id","property_value":"458888","operator":"gt"}},{{"property_name":"id","property_value":"460000","operator":"lte"}}]&interval=monthly"#, from_str, to_str));
}
