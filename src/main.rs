extern crate currency;

use std::io;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
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

fn normalize_input<T: Iterator<Item=String>>(lines: T) -> Vec<Record> {
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
    records_raw.into_iter().map(|record| {
        let debtors: Vec<String> =
            if record.debtors.is_empty() {
                &participants
            } else {
                &record.debtors
            }.iter()
            .filter(|debtor| debtor != &&record.creditor)
            .cloned().collect();
        Record { debtors: debtors, .. record }
    }).collect()
}

fn run<T: Iterator<Item=String>>(lines: T) -> Vec<Debt> {
    let records = normalize_input(lines);
    let debts = records.into_iter().flat_map(|record| {
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
    let deduplicated: Vec<Debt> = debts.fold(HashMap::new(), |mut memo, elem| {
        memo.entry(elem.debtor.clone()).or_insert(HashMap::new())
            .entry(elem.creditor.clone()).or_insert(vec![]).push(elem.amount);
        memo
    })
    .into_iter()
    .flat_map(|(debtor, by_creditor)| {
        by_creditor.into_iter().map(|(creditor, amounts)| {
            Debt {
                debtor: debtor.clone(),
                amount: sum(amounts.into_iter()),
                creditor: creditor,
            }
        })
        .collect::<Vec<_>>()
    })
    .collect();

    let mut by_debtor: Vec<(String, Vec<Debt>)> =
        deduplicated.into_iter().fold(HashMap::new(), |mut memo, elem| {
            memo.entry(elem.debtor.clone()).or_insert(vec![]).push(elem);
            memo
        }).into_iter().collect();
    by_debtor.sort_by_key(|elem| {
        sum(elem.1.iter().map(|e| e.amount.clone())).value().clone()
    });

    vec![]
    // debts.collect()

    // for record in records {
    //     println!("{:?}", record);
    //     print!("{}", record.creditor);
    //     print!(" {}", record.amount);
    //     print!(" {}", record.amount / 2);
    //     println!(" {:?}", record.debtors);
    // }
}

fn sum<I: IntoIterator<Item=Currency>>(amounts: I) -> Currency {
    amounts.into_iter().fold(Currency::new(), |memo, elem| memo + elem)
}

// fn sum(amounts: &[Currency]) -> Currency {
//     amounts.iter().fold(Currency::new(), |memo, elem| memo + elem)
// }


#[cfg(test)]
mod tests {
    // use currency::Currency;
    use super::Debt;
    use super::run;

    #[test]
    fn test_multi() {
        println!("{}", INPUT_1);
        assert_eq!(INPUT_1.lines().count(), 4);
        let actual = run(INPUT_1.lines().map(|x| x.to_owned()));
        let expected = vec![
            Debt { debtor: "c".into(), amount: "550".parse().unwrap(), creditor: "a".into() },
            Debt { debtor: "c".into(), amount: "100".parse().unwrap(), creditor: "b".into() },
        ];
        println!("{:?}", expected);
        for x in expected.iter() {
            println!("{}", x);
        }
        assert_eq!(actual, expected);
    }

    // fn mk_debt(debt: &str) -> Debt {
    //     let mut itr = debt.split_whitespace();
    //     let d = itr.next().unwrap();
    //     let amt = itr.next().unwrap();
    //     let c = itr.next().unwrap();
    //     Debt {
    //         debtor: d.into(),
    //         amount: amt.parse().unwrap(),
    //         creditor: c.into() }
    // }

    const INPUT_1: &'static str = "\
        a 1200 a b c\n\
        b 600 a b c\n\
        b 200 b c\n\
        c 100 a c\n\
        ";

    #[test]
    fn test_main() {
        let test_data = "S 8.49 F E\nE 16.99";
        run(test_data.lines().map(|s| s.to_owned()));
        // let actual = run(test_data.lines().map(|s| s.to_owned()));
        // println!("");
        // println!("input:\n{}\n", test_data);
        // for debt in actual {
        //     println!("{}", debt);
        // }
        // assert!(false);
    }

    #[test]
    fn test_2() {
        // let test_data =
        //     "ada 200
        //     kolle 100
        //     leila 20
        //     billy 10";
        // let actual = run(test_data.lines().map(|s| s.to_owned()));

        // // println!("\ninput:\n{}\n", test_data);

        // // for debt in &actual {
        // //     println!("{}", debt);
        // // }

        // fn mk_debt(debtor: &str, amount: &str, creditor: &str) -> Debt {
        //     Debt {
        //         debtor: debtor.into(),
        //         amount: Currency::from_str(amount).unwrap(),
        //         creditor: creditor.into()
        //     }
        // }

        // let expected = vec![
        //     mk_debt("billy", "72.5", "ada"),
        //     mk_debt("leila", "45", "ada"),
        //     mk_debt("leila", "17.5", "kolle"),
        // ];

        // assert_eq!(actual, expected);

        // DebtResolution("billy", "ada", BigDecimal("72.5")),
        // DebtResolution("leila", "ada", BigDecimal("45")),
        // DebtResolution("leila", "kolle", BigDecimal("17.5"))
    }
}
