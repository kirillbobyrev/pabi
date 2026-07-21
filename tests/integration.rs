use predicates::boolean::PredicateBooleanExt;
use predicates::str::{contains, is_match};

#[test]
fn uci_setup() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("uci\n") // Write the uci command to stdin
            .assert()
            .success()
            .stdout(
                contains("id name")
                    .and(contains("id author"))
                    .and(contains("uciok")),
            ),
    );
}

#[test]
fn uci_bestmove_with_node_limit() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("uci\nisready\nposition startpos moves e2e4 e7e5\ngo nodes 500\nquit\n")
            .assert()
            .success()
            .stdout(contains("readyok").and(is_match(r"bestmove [a-h][1-8][a-h][1-8]").unwrap())),
    );
}

#[test]
fn uci_finds_backrank_mate() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("position fen 6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1\ngo nodes 3000\nquit\n")
            .assert()
            .success()
            .stdout(contains("bestmove a1a8")),
    );
}

#[test]
fn uci_stop_terminates_infinite_search() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("position startpos\ngo infinite\nstop\nquit\n")
            .assert()
            .success()
            .stdout(is_match(r"bestmove [a-h][1-8][a-h][1-8]").unwrap()),
    );
}

#[test]
fn uci_go_movetime() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("position startpos\ngo movetime 100\nquit\n")
            .assert()
            .success()
            .stdout(is_match(r"bestmove [a-h][1-8][a-h][1-8]").unwrap()),
    );
}

#[test]
fn uci_rejects_illegal_moves() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin("position startpos moves e2e5\nquit\n")
            .assert()
            .success()
            .stdout(contains("info string invalid position")),
    );
}

#[test]
fn uci_bestmove_none_when_no_legal_moves() {
    // Black is checkmated; there is nothing to search.
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin(
            "position fen rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 4 3\ngo nodes \
             10\nquit\n",
        )
        .assert()
        .success()
        .stdout(contains("bestmove (none)")),
    );
}

#[test]
fn openbench_output() {
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");
    let _ = cmd.arg("bench");

    drop(
        cmd.assert()
            .success()
            .stdout(is_match(r"(?m)^\d+ nodes \d+ nps$").unwrap()),
    );
}

#[test]
fn uci_reuses_tree_across_continuing_searches() {
    // Two searches on the same continuing game exercise the tree-reuse path in
    // a single process; both must still produce a legal bestmove.
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin(
            "position startpos moves e2e4\ngo nodes 800\n\
             position startpos moves e2e4 e7e5\ngo nodes 800\nquit\n",
        )
        .assert()
        .success()
        .stdout(
            is_match(r"(?s)bestmove [a-h][1-8][a-h][1-8].*bestmove [a-h][1-8][a-h][1-8]").unwrap(),
        ),
    );
}

#[test]
fn uci_loads_syzygy_tablebase() {
    let tablebase_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/syzygy");
    let mut cmd = assert_cmd::cargo_bin_cmd!("pabi");

    drop(
        cmd.write_stdin(format!(
            "setoption name SyzygyTablebase value {tablebase_dir}\nstate\nquit\n"
        ))
        .assert()
        .success()
        .stdout(
            contains("loaded Syzygy tablebases")
                .and(contains("option SyzygyTablebase loaded, up to 3 pieces")),
        ),
    );
}
