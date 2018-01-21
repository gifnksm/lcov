use super::Record;
use super::Record::*;

fn check_parse_ok(s: &str, rec: &Record) {
    assert_eq!(s.parse::<Record>().unwrap(), *rec);
    assert_eq!(s.trim().parse::<Record>().unwrap(), *rec);
    assert_eq!(format!("{}\r\n", s.trim()).parse::<Record>().unwrap(), *rec);
    assert_eq!(rec.to_string(), s);
}


#[test]
fn test_name() {
    fn check_ok(s: &str) {
        check_parse_ok(&format!("TN:{}\n", s), &TestName { name: s.into() });
    }
    check_ok("foo");
    check_ok("foo:bar");
    check_ok("foo:bar,baz");
}

#[test]
fn source_file() {
    fn check_ok(s: &str) {
        check_parse_ok(&format!("SF:{}\n", s), &SourceFile { path: s.into() })
    }
    check_ok("/foo/bar/baz");
    check_ok("C:/foo/bar/baz");
    check_ok(r"C:\foo\bar\baz");
}

#[test]
fn function_name() {
    fn check_ok(name: &str, line: u32) {
        check_parse_ok(
            &format!("FN:{},{}\n", line, name),
            &FunctionName {
                name: name.into(),
                start_line: line,
            },
        )
    }
    check_ok("hogehoge", 3);
    check_ok("3,5", 1);
}

#[test]
fn function_data() {
    fn check_ok(name: &str, count: u64) {
        check_parse_ok(
            &format!("FNDA:{},{}\n", count, name),
            &FunctionData {
                name: name.into(),
                count,
            },
        );
    }
    check_ok("hogehoge", 12345);
    check_ok("hoge,hoge", 98765);
}

#[test]
fn functions_found_hit() {
    fn check_ok(n: u64) {
        check_parse_ok(&format!("FNF:{}\n", n), &FunctionsFound { found: n });
        check_parse_ok(&format!("FNH:{}\n", n), &FunctionsHit { hit: n });
    }
    check_ok(0);
    check_ok(100);
    check_ok(u64::max_value());
}

#[test]
fn branch_data() {
    fn check_ok(line: u32, block: u32, branch: u32, taken: Option<u64>) {
        let s = if let Some(taken) = taken {
            format!("BRDA:{},{},{},{}\n", line, block, branch, taken)
        } else {
            format!("BRDA:{},{},{},-\n", line, block, branch)
        };
        check_parse_ok(
            &s,
            &BranchData {
                line,
                block,
                branch,
                taken,
            },
        );
    }
    check_ok(10, 20, 30, Some(40));
    check_ok(100, 200, 300, None);
}

#[test]
fn branches_found_hit() {
    fn check_ok(n: u64) {
        check_parse_ok(&format!("BRF:{}\n", n), &BranchesFound { found: n });
        check_parse_ok(&format!("BRH:{}\n", n), &BranchesHit { hit: n });
    }
    check_ok(0);
    check_ok(100);
    check_ok(u64::max_value());
}

#[test]
fn line_data() {
    fn check_ok(line: u32, count: u64, checksum: Option<String>) {
        let s = if let Some(ref checksum) = checksum {
            format!("DA:{},{},{}\n", line, count, checksum)
        } else {
            format!("DA:{},{}\n", line, count)
        };
        check_parse_ok(
            &s,
            &LineData {
                line,
                count,
                checksum,
            },
        );
    }
    check_ok(10, 20, None);
    check_ok(u32::max_value(), u64::max_value(), Some("hogehoge".into()));
    check_ok(
        u32::max_value(),
        u64::max_value(),
        Some("foo,bar,baz".into()),
    );
}

#[test]
fn lines_found_hit() {
    fn check_ok(n: u64) {
        check_parse_ok(&format!("LF:{}\n", n), &LinesFound { found: n });
        check_parse_ok(&format!("LH:{}\n", n), &LinesHit { hit: n });
    }
    check_ok(0);
    check_ok(100);
    check_ok(u64::max_value());
}

#[test]
fn end_of_record() {
    check_parse_ok("end_of_record\n", &EndOfRecord);
}
