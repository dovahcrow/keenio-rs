extern crate keen;
extern crate chrono;
extern crate rustc_serialize;
extern crate docopt;

use keen::{KeenClient, Filter, Metric, TimeFrame};
use std::io::Read;
use std::{time, env};
use chrono::*;


static USAGE: &'static str = "
Usage:
  main <from> <to>
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_from: i64,
    arg_to: i64,
}


fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let key = env::var("KEEN_READ_KEY").unwrap();
    let proj = env::var("KEEN_PROJECT_ID").unwrap();

    let mut client = KeenClient::new(&key, &proj);

    client.timeout(time::Duration::new(30, 0));
    let metric = Metric::CountUnique("ip_address".into());

    let mut q = client.query(metric.clone(),
                             "strikingly_pageviews".into(),
                             TimeFrame::Absolute(UTC::now() - Duration::hours(10),
                                                 UTC::now() - Duration::hours(1)));
    q.filter(Filter::gt("pageId", args.arg_from));
    q.filter(Filter::lt("pageId", args.arg_to));
    q.group_by("normalized_referrer");
    q.group_by("ip_geo_info.country");
    q.group_by("parsed_user_agent.os.family");
    q.group_by("pageId");
    match q.data() {
        Ok(mut d) => {
            let mut s = String::new();
            let _ = d.read_to_string(&mut s);
            println!("{:?}", s);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
