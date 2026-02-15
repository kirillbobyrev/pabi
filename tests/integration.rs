use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;

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

// #[test]
// #[ignore]
// fn openbench_output() {
//     let mut cmd = Command::cargo_bin(BINARY_NAME).expect("Binary should be
// built");     let _ = cmd.arg("bench");

//     drop(
//         cmd.assert()
//             .stdout(is_match(r"^\d+ nodes \d+ nps$").unwrap())
//             .success(),
//     );
// }
