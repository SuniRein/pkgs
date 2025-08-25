test:
    cargo nextest run

coverage:
    cargo llvm-cov nextest --open --ignore-filename-regex logger/output\.rs

coverage-tarpaulin:
    cargo tarpaulin --out html
    xdg-open tarpaulin-report.html
