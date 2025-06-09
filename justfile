run:
    cargo run --release -- run --settings-file benchmark-lite.json
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html

run-full:
    cargo run --release -- run --settings-file benchmark.json
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html

report:
    cargo run --release -- report --report-file target/report.json --report-html-file target/report.html
