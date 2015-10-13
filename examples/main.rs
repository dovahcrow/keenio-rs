extern crate keen;
extern crate chrono;

use keen::*;
use std::io::Read;
use chrono::*;

fn main() {
    let cl = KeenClient::new("your keen io api key", "you project id");
    let m = Metric::CountUnique("metric1".into());
    let c = "collection_name".into();
    let t = TimeFrame::Abs(UTC::now() - Duration::days(2), UTC::now());
    let mut q = cl.query(m, c, t);
    q.add_group("group1");
    q.add_group("group2");
    q.add_filter(Filter::new("id", Operator::Gt, "458888"));
    q.add_filter(Filter::new("id", Operator::Lte, "460000"));
    q.interval(Interval::Monthly);
    let mut resp = q.data().unwrap();
    let mut s = String::new();
    let _ = resp.read_to_string(&mut s);
    println!("url is: {}", q.url());
    println!("data is: {}", s);
}
