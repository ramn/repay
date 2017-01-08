extern crate ramn_currency;

use std::io;
use std::io::prelude::*;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use ramn_currency::Currency;


#[derive(Debug, PartialEq, Eq)]
struct Record {
    creditor: String,
    amount: Currency,
    debtors: BTreeSet<String>,
}

struct Records {
    records: Vec<Record>,
}

#[derive(Debug, Eq, PartialEq)]
struct Debt {
    debtor: String,
    amount: Currency,
    creditor: String,
}

fn main() {
    let stdin = io::stdin();
    for debt in run(stdin.lock().lines().map(|s| s.unwrap())) {
        println!("{}", debt);
    }
}

fn normalize_input<T: IntoIterator<Item=String>>(lines: T) -> Vec<Record> {
    let records: Vec<Record> = lines.into_iter()
        .filter(|s| !s.is_empty())
        .map(|line| {
            let mut tokens = line.split_whitespace();
            Record {
                creditor: tokens.next().unwrap().into(),
                amount: tokens.next().unwrap().parse().unwrap(),
                debtors: tokens.map(|s| s.to_owned()).collect(),
            }
        }).collect();
    let participants: BTreeSet<String> =
        records.iter().fold(BTreeSet::new(), |mut memo, elem| {
            memo.insert(elem.creditor.to_owned());
            memo.extend(elem.debtors.clone());
            memo
        });
    records.into_iter().map(|record| {
        let debtors: BTreeSet<String> =
            if record.debtors.is_empty() {
                &participants
            } else {
                &record.debtors
            }.iter()
            .cloned().collect();
        Record { debtors: debtors, .. record }
    }).collect()
}

fn run<T: IntoIterator<Item=String>>(lines: T) -> Vec<Debt> {
    Records::new(lines).calc_debt_resolution()
}

fn sum<'a, I>(amounts: I) -> Currency
    where I: IntoIterator<Item=&'a Currency> {
    amounts.into_iter().fold(Currency::new(), |memo, elem| memo + elem)
}

fn parse<T>(s: &str) -> T
    where T: FromStr, T::Err: std::fmt::Debug {
    s.parse().unwrap()
}

fn str2btree_set(xs: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    out.extend(xs.split_whitespace().map(|x| x.to_owned()));
    out
}

impl Record {
    #[allow(dead_code)]
    fn new(creditor: &str, amount: &str, debtors_init: &str) -> Record {
        Record {
            creditor: creditor.into(),
            amount: parse(amount),
            debtors: str2btree_set(debtors_init),
        }
    }
}

impl Records {
    fn new<T: IntoIterator<Item=String>>(records_init: T) -> Records {
        Records { records: normalize_input(records_init) }
    }

    fn unique_people(&self) -> BTreeSet<String> {
        self.records.iter().fold(BTreeSet::new(), |mut memo, elem| {
            memo.insert(elem.creditor.clone());
            memo
        })
    }

    fn calc_share_per_group(&self) -> BTreeMap<BTreeSet<String>, Currency> {
        let mut share_per_group =
            self.records.iter().fold(BTreeMap::new(), |mut memo, elem| {
                memo.entry(elem.debtors.clone()).or_insert(Currency::new());
                memo
            });
        for (group, share) in share_per_group.iter_mut() {
            let amounts: Vec<_> = self.records.iter()
                .filter(|&record| record.debtors == *group)
                .map(|record| record.amount.clone())
                .collect();
            *share = sum(amounts.iter()) / group.len();
        };
        share_per_group
    }

    fn calc_share_per_person(&self) -> BTreeMap<String, Currency> {
        let share_per_group = self.calc_share_per_group();
        self.unique_people().into_iter().map(|person| {
            let share = sum(
                share_per_group.iter()
                .filter(|&(key, _value)| key.contains(&person))
                .map(|(_key, value)| value));
            (person, share)
        }).collect()
    }

    fn calc_expenses_per_person(&self) -> BTreeMap<String, Currency> {
        self.records.iter().fold(BTreeMap::new(), |mut memo, record| {
            {
                let amount = memo.entry(record.creditor.clone())
                    .or_insert(Currency::new());
                *amount = amount.clone() + &record.amount;
            }
            memo
        })
    }

    fn calc_debt_per_person(&self) -> BTreeMap<String, Currency> {
        let share_per_person = self.calc_share_per_person();
        let expenses_per_person = self.calc_expenses_per_person();
        share_per_person.into_iter().map(|(person, share)| {
            let debt = share - expenses_per_person.get(&person)
                .unwrap_or(&Currency::new());
            (person, debt)
        }).collect()
    }

    fn calc_debt_resolution(&self) -> Vec<Debt> {
        let debt_per_person = self.calc_debt_per_person();

        let debt_per_person_by_debt = {
            let mut d: Vec<(String, Currency)> = debt_per_person.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .filter(|&(_, ref v)| v > &parse("0"))
                .collect();
            d.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            d
        };
        let mut expense_per_person = {
            let mut d: Vec<(String, Currency)> = debt_per_person.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .filter(|&(_, ref v)| v < &parse("0"))
                .collect();
            d.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            d
        };
        debt_per_person_by_debt.into_iter()
            .flat_map(|(person, debt)| self.resolve_for_person(
                    &person,
                    &debt,
                    &mut expense_per_person))
            .collect()
    }

    fn resolve_for_person(
        &self,
        person: &str,
        debt: &Currency,
        expense_per_person: &mut [(String, Currency)]
    ) -> Vec<Debt> {
        let mut debt = debt.clone();
        let mut payouts = vec![];
        let zero = parse("0");
        while debt > zero {
            let pos_opt = expense_per_person.iter()
                .position(|x| x.0 != person && &x.1 < &zero);
            if let Some(pos) = pos_opt {
                let (ref creditor, ref mut expense) = expense_per_person[pos];
                let remainder = &debt + expense.clone();
                if &remainder >= &zero {
                    debt = remainder;
                    payouts.push((creditor.clone(), expense.clone() * -1));
                    *expense = zero.clone();
                } else if &remainder < &zero {
                    *expense = remainder;
                    payouts.push((creditor.clone(), debt.clone()));
                    debt = zero.clone();
                }
            } else {
                unreachable!("Should always find a creditor");
            }
        }
        payouts.into_iter().map(|(creditor, amount)|
            Debt {
                debtor: person.to_owned(),
                amount: amount,
                creditor: creditor
            }
       ).collect()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let debtors: Vec<_> = self.debtors.iter().map(|x| x.as_str()).collect();
        write!(f, "Creditor: {}, Amount: {}, Debtors {}",
               self.creditor,
               self.amount,
               debtors.as_slice().join(", "))
    }
}

impl fmt::Display for Debt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} owes {} {}", self.debtor, self.creditor, self.amount)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use super::Debt;
    use super::Record;
    use super::Records;
    use super::normalize_input;
    use super::parse;
    use super::run;
    use super::str2btree_set;

    #[test]
    fn test_str2btree_set() {
        let actual = str2btree_set("a b c");
        let mut expected = BTreeSet::new();
        expected.insert("a".into());
        expected.insert("b".into());
        expected.insert("c".into());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_normalize_input() {
        let actual = normalize_input(get_input());
        let expected = vec![
            Record::new("a", "1200", "a b c"),
            Record::new("b", "600", "a b c"),
            Record::new("b", "200", "b c"),
            Record::new("c", "100", "a c"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_normalize_input_no_debtors() {
        let actual = normalize_input(
            "a 100\nb 100\n\n"
            .lines().map(|x|x.to_owned()));
        let expected = vec![
            Record::new("a", "100", "a b"),
            Record::new("b", "100", "a b"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_share_per_group() {
        let records = Records::new(get_input());
        let actual = records.calc_share_per_group();
        assert_eq!(3, actual.len());
        let expected = vec![
            (str2btree_set("a b c"), parse("600")),
            (str2btree_set("a c"), parse("50")),
            (str2btree_set("b c"), parse("100")),
        ].into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_share_per_person() {
        let records = Records::new(get_input());
        let actual = records.calc_share_per_person();
        let expected = vec![
            ("a".into(), parse("650")),
            ("b".into(), parse("700")),
            ("c".into(), parse("750")),
        ].into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_expenses_per_person() {
        let records = Records::new(get_input());
        let actual = records.calc_expenses_per_person();
        let expected = vec![
            ("a".into(), parse("1200")),
            ("b".into(), parse("800")),
            ("c".into(), parse("100")),
        ].into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_debt_per_person() {
        let records = Records::new(get_input());
        let actual = records.calc_debt_per_person();
        let expected = vec![
            ("a".into(), parse("-550")),
            ("b".into(), parse("-100")),
            ("c".into(), parse("650")),
        ].into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_debt_resolution() {
        let records = Records::new(get_input());
        let actual = records.calc_debt_resolution();
        let expected = vec![
            Debt {
                debtor: "c".into(), amount: parse("550"), creditor: "a".into()
            },
            Debt {
                debtor: "c".into(), amount: parse("100"), creditor: "b".into()
            },
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_run() {
        let actual = run(get_input());
        let expected = vec![
            Debt {
                debtor: "c".into(), amount: parse("550"), creditor: "a".into()
            },
            Debt {
                debtor: "c".into(), amount: parse("100"), creditor: "b".into()
            },
        ];
        assert_eq!(actual, expected);
    }

    fn get_input() -> Vec<String> {
        INPUT_1.lines().map(|x| x.to_owned()).collect()
    }

    const INPUT_1: &'static str = "\
        a 1200 a b c\n\
        b 600 a b c\n\
        b 200 b c\n\
        c 100 a c\n\
        ";

    #[test]
    fn test_input() {
        assert_eq!(INPUT_1.lines().count(), 4);
    }
}
