run:
    cargo run --release -- run --settings-file benchmark-lite.json
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html

run-full:
    cargo run --release -- run --settings-file benchmark.json
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html

run-full-hdd:
    cargo run --release -- run --settings-file benchmark.json --test-file /run/media/dvk/dvk-hdd-big2/temp/2025-06-09/test_file --report-file target/report-hdd.json
    cargo run --release -- report --report-file target/report-hdd.json --report-html-file target/report-hdd.html

report:
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html
