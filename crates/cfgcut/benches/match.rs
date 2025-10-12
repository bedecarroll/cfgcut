use std::path::PathBuf;

use cfgcut::{Anonymization, CommentHandling, OutputMode, RunRequest, run};
use criterion::{Criterion, criterion_group, criterion_main};

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

fn bench_ios_match(c: &mut Criterion) {
    let path = fixture_path("cisco_ios/sample.conf");
    c.bench_function("ios_interface_match", |b| {
        b.iter(|| {
            let request = RunRequest::builder()
                .matches(vec!["interface GigabitEthernet1|>>|".to_string()])
                .comment_handling(CommentHandling::Exclude)
                .output_mode(OutputMode::Quiet)
                .anonymization(Anonymization::Disabled)
                .inputs(vec![path.clone()])
                .build();
            let result = run(&request).expect("run succeeds");
            assert!(result.matched);
        });
    });
}

fn bench_junos_match(c: &mut Criterion) {
    let path = fixture_path("juniper_junos/sample.conf");
    c.bench_function("junos_interfaces_subtree", |b| {
        b.iter(|| {
            let request = RunRequest::builder()
                .matches(vec!["interfaces|>>|".to_string()])
                .comment_handling(CommentHandling::Exclude)
                .output_mode(OutputMode::Quiet)
                .anonymization(Anonymization::Disabled)
                .inputs(vec![path.clone()])
                .build();
            let result = run(&request).expect("run succeeds");
            assert!(result.matched);
        });
    });
}

criterion_group!(benches, bench_ios_match, bench_junos_match);
criterion_main!(benches);
