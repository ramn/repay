Repay
=====

CLI for calculating repayments

Example
-------

    repay <<HERE
    a 150
    b 300
    c 100 c a
    HERE
    c owes b 100.00
    a owes b 50.00

Links
-----

 * [Crate](https://crates.io/crates/repay)
 * [Documentation](https://docs.rs/repay)

How to install
--------------
Download from
[https://github.com/ramn/repay/releases](https://github.com/ramn/repay/releases)
or run `cargo install repay`.

TODO
----

  * Support semicolon as separator in addition to newline
  * Support -h flag
  * Support comments the way Bash does it, from #, skip the rest of the line.
