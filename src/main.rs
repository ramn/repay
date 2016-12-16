extern crate currency;

use std::io;
use std::io::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use std::fmt;

use currency::Currency;


#[derive(Debug)]
struct Record {
    creditor: String,
    amount: Currency,
    debtors: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct Debt {
    debtor: String,
    amount: Currency,
    creditor: String,
}

impl fmt::Display for Debt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} owes {} {}", self.debtor, self.creditor, self.amount)
    }
}


fn main() {
    let stdin = io::stdin();
    run(stdin.lock().lines().map(|s| s.unwrap()));
}

fn run<T: Iterator<Item=String>>(lines: T) -> Vec<Debt> {
    let records_raw: Vec<Record> = lines.map(|line| {
        let mut tokens = line.split_whitespace();
        Record {
            creditor: tokens.next().unwrap().into(),
            amount: tokens.next().unwrap().parse().unwrap(),
            debtors: tokens.map(|s| s.to_owned()).collect(),
        }
    }).collect();

    let participants: Vec<String> =
        records_raw.iter().fold(HashSet::new(), |mut memo, elem| {
            memo.insert(elem.creditor.to_owned());
            memo.extend(elem.debtors.clone());
            memo
        }).into_iter().collect();


    let records = records_raw.into_iter().map(|record| {
        let debtors: Vec<String> =
            if record.debtors.is_empty() {
                &participants
            } else {
                &record.debtors
            }.iter()
            .filter(|debtor| debtor != &&record.creditor)
            .cloned().collect();
        Record { debtors: debtors, .. record }
    });

    let debts = records.flat_map(|record| {
        let Record { creditor, amount, debtors } = record;
        assert!(debtors.len() > 0);
        let share = amount / debtors.len();
        debtors.into_iter().map(|d|
            Debt {
                debtor: d,
                amount: share.clone(),
                creditor: creditor.clone()
            }
        ).collect::<Vec<_>>()
    });

    debts.iter().fold(HashMap::new(), |memo, elem| {
        memo.entry(elem.debtor).or_insert(HashMap::new())
            .entry(elem.creditor).or_insert(vec![]).push(elem);
        memo
    });
    debts.collect()

    // for record in records {
    //     println!("{:?}", record);
    //     print!("{}", record.creditor);
    //     print!(" {}", record.amount);
    //     print!(" {}", record.amount / 2);
    //     println!(" {:?}", record.debtors);
    // }
}


#[cfg(test)]
mod tests {
    use currency::Currency;
    use super::Debt;
    use super::run;

    #[test]
    fn test_main() {
        let test_data = "S 8.49 F E\nE 16.99";
        let actual = run(test_data.lines().map(|s| s.to_owned()));
        println!("");
        println!("input:\n{}\n", test_data);
        for debt in actual {
            println!("{}", debt);
        }
        // assert!(false);
    }

    #[test]
    fn test_2() {
        let test_data =
            "ada 200
            kolle 100
            leila 20
            billy 10";
        let actual = run(test_data.lines().map(|s| s.to_owned()));

        println!("\ninput:\n{}\n", test_data);

        for debt in &actual {
            println!("{}", debt);
        }

        fn mk_debt(debtor: &str, amount: &str, creditor: &str) -> Debt {
            Debt {
                debtor: debtor.into(),
                amount: Currency::from_str(amount).unwrap(),
                creditor: creditor.into()
            }
        }

        let expected = vec![
            mk_debt("billy", "72.5", "ada"),
            mk_debt("leila", "45", "ada"),
            mk_debt("leila", "17.5", "kolle"),
        ];

        assert_eq!(actual, expected);
        // DebtResolution("billy", "ada", BigDecimal("72.5")),
        // DebtResolution("leila", "ada", BigDecimal("45")),
        // DebtResolution("leila", "kolle", BigDecimal("17.5"))
    }
}
