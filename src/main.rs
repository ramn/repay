//! Example usage:
//!
//!
//! ```
//! $ cargo install repay
//!
//! $ repay <<HERE
//! a 150
//! b 300
//! c 100 c a
//! HERE
//! c owes b 100.00
//! a owes b 50.00
//! ```

extern crate num;

mod money;

use std::io;
use std::io::prelude::*;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::fmt;

use money::{Money};

#[derive(Debug, PartialEq, Eq)]
struct Record {
    creditor: String,
    amount: Money,
    debtors: BTreeSet<String>,
}

struct Records {
    records: Vec<Record>,
}

#[derive(Debug, Eq, PartialEq)]
struct Debt {
    debtor: String,
    amount: Money,
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
                amount: money::parse(tokens.next().unwrap()),
                debtors: tokens.map(|s| s.to_owned()).collect(),
            }
        }).collect();
    let participants: BTreeSet<String> =
        records.iter().fold(BTreeSet::new(), |mut memo, elem| {
            memo.insert(elem.creditor.to_owned());
            memo.extend(elem.debtors.clone());
            memo
        });
    let creditors: BTreeSet<String> =
        records.iter().map(|rec| rec.creditor.clone()).collect();
    // We need records also for persons with no expenses so they become part of
    // the calculation further down.
    let debtors_with_no_credit = participants.difference(&creditors)
        .map(|debtor| Record {
            creditor: debtor.clone(),
            amount: money::zero(),
            debtors: BTreeSet::new(),
        });
    records.into_iter().chain(debtors_with_no_credit).map(|record| {
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

fn sum<'a, I>(amounts: I) -> Money
    where I: IntoIterator<Item=&'a Money> {
    amounts.into_iter().fold(money::zero(), |memo, elem| memo + elem)
}

impl Records {
    fn new<T: IntoIterator<Item=String>>(records_init: T) -> Records {
        Records { records: normalize_input(records_init) }
    }

    fn calc_expenses_per_person2<R: AsRef<Record>>(
        records: &[R]
    ) -> BTreeMap<String, Money> {
        records.iter().fold(BTreeMap::new(), |mut memo, record| {
            let record = record.as_ref();
            {
                let amount = memo.entry(record.creditor.clone())
                    .or_insert_with(money::zero);
                *amount = amount.clone() + &record.amount;
            }
            memo
        })
    }

    fn calc_expenses_per_person_and_group(
        &self
    ) -> BTreeMap<BTreeSet<String>, BTreeMap<String, Money>> {
        self.records_by_group().iter().map(|(group, records)| {
            (group.clone(), Self::calc_expenses_per_person2(records))
        }).collect()
    }

    fn calc_share(records: &[&Record], group_size: usize) -> Money {
        sum(records.iter().map(|r| &r.amount)) / money::from(group_size)
    }

    fn calc_share_per_person_and_group(
        &self
    ) -> BTreeMap<BTreeSet<String>, BTreeMap<String, Money>> {
        self.records_by_group().iter().map(|(group, records)| {
            let share = Self::calc_share(records, group.len());
            let share_per_person = records.iter()
                .flat_map(|rec| rec.debtors.iter()
                    .map(|debtor| (debtor.clone(), share.clone())))
                .collect();
            (group.clone(), share_per_person)
        }).collect()
    }

    fn records_by_group(&self) -> BTreeMap<BTreeSet<String>, Vec<&Record>> {
        self.records.iter().fold(BTreeMap::new(), |mut memo, elem| {
            memo.entry(elem.debtors.clone())
                .or_insert_with(||vec![])
                .push(elem);
            memo
        })
    }

    fn expenses_creditor_not_part_of_group(
        &self
    ) -> BTreeMap<String, Money> {
        self.records.iter()
            .filter(|&rec| !rec.debtors.contains(&rec.creditor))
            .map(|rec| (rec.creditor.clone(), rec.amount.clone() * money::from(-1)))
            .collect()
    }

    fn calc_debt_per_person(&self) -> BTreeMap<String, Money> {
        let share_per_person_and_group =
            self.calc_share_per_person_and_group();
        let expenses_per_person_group =
            self.calc_expenses_per_person_and_group();
        share_per_person_and_group.into_iter()
            .map(|(group, share_per_person)| {
                let expenses_per_person = &expenses_per_person_group[&group];
                share_per_person.into_iter().map(|(person, share)| {
                    let debt = share - expenses_per_person.get(&person)
                        .unwrap_or(&money::zero());
                    (person.clone(), debt)
                }).collect::<BTreeMap<String, Money>>()
            })
            .chain(vec![self.expenses_creditor_not_part_of_group()].into_iter())
            .fold(BTreeMap::new(), |mut memo, debt_per_person| {
                for (person, debt) in debt_per_person {
                    let debt_acc = memo.entry(person)
                        .or_insert_with(money::zero);
                    *debt_acc = debt_acc.clone() + debt;
                }
                memo
            })
    }

    fn calc_debt_resolution(&self) -> Vec<Debt> {
        let debt_per_person = self.calc_debt_per_person();
        let debt_per_person_by_debt = {
            let mut d: Vec<(String, Money)> = debt_per_person.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .filter(|&(_, ref v)| v > &money::zero())
                .collect();
            d.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            d
        };
        let mut expense_per_person = {
            let mut d: Vec<(String, Money)> = debt_per_person.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .filter(|&(_, ref v)| v < &money::zero())
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
        debt: &Money,
        expense_per_person: &mut [(String, Money)]
    ) -> Vec<Debt> {
        let mut debt = debt.clone();
        let mut payouts = vec![];
        let zero: Money = money::zero();
        let mut last_debt = money::zero();
        while &debt != &last_debt && &debt != &zero {
            last_debt = debt.clone();
            let pos_opt = expense_per_person.iter()
                .position(|x| x.0 != person && &x.1 < &zero);
            if let Some(pos) = pos_opt {
                let (ref creditor, ref mut expense) = expense_per_person[pos];
                let remainder = &debt + expense.clone();
                if &remainder >= &zero {
                    debt = remainder;
                    payouts.push(
                        (creditor.clone(), expense.clone() * money::from(-1)));
                    *expense = money::zero();
                } else if &remainder < &zero {
                    *expense = remainder;
                    payouts.push((creditor.clone(), debt.clone()));
                    debt = money::zero();
                }
            } else {
                break;
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
        let amount = money::to_float(&self.amount);
        write!(f, "{} owes {} {:.2}", self.debtor, self.creditor, amount)
    }
}

impl AsRef<Record> for Record {
    fn as_ref(&self) -> &Record { self }
}


#[cfg(test)]
mod tests {
    use std::fmt;
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use super::Debt;
    use super::Record;
    use super::Records;
    use super::normalize_input;
    use super::run;
    use super::money;

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
    fn test_calc_share_per_person_and_group() {
        let records = Records::new(get_input());
        let actual = records.calc_share_per_person_and_group();
        assert_eq!(3, actual.len());
        let expected = vec![
            (str2btree_set("a b c"),
                vec![
                    ("a".into(), parse("600")),
                    ("b".into(), parse("600")),
                    ("c".into(), parse("600"))
                ].into_iter().collect()),
            (str2btree_set("a c"),
                vec![
                    ("a".into(), parse("50")),
                    ("c".into(), parse("50")),
                ].into_iter().collect()),
            (str2btree_set("b c"),
                vec![
                    ("b".into(), parse("100")),
                    ("c".into(), parse("100")),
                ].into_iter().collect()),
        ].into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calc_expenses_per_person_and_group() {
        let records = Records::new(get_input());
        let actual = records.calc_expenses_per_person_and_group();
        let expected = vec![
            (str2btree_set("a b c"),
                vec![
                    ("a".into(), parse("1200")),
                    ("b".into(), parse("600")),
                ].into_iter().collect()),
            (str2btree_set("a c"),
                vec![
                    ("c".into(), parse("100")),
                ].into_iter().collect()),
            (str2btree_set("b c"),
                vec![
                    ("b".into(), parse("200")),
                ].into_iter().collect()),
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

    #[test]
    fn test_only_one_expenser() {
        let actual = run(to_input("a 100 b"));
        let expected = vec![mk_debt("b 100 a")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_a_to_b_and_b_to_c() {
        let actual = run(to_input("a 100 b\nb 50 c"));
        let expected = vec![mk_debt("b 50 a"), mk_debt("c 50 a")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fractions() {
        let actual = stringify(run(to_input("a 33.33 b c\nb 12.33 a c")));
        let expected = stringify(vec![
            mk_debt("c 22.83 a"),
            mk_debt("b 4.33 a"),
        ]);
        assert_eq!(expected, actual);
    }

    fn to_input(s: &str) -> Vec<String> {
        s.lines().map(|x| x.to_owned()).collect()
    }

    fn get_input() -> Vec<String> {
        to_input(INPUT_1)
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

    fn mk_debt(debt: &str) -> Debt {
        let mut itr = debt.split_whitespace();
        let d = itr.next().unwrap();
        let amt = itr.next().unwrap();
        let c = itr.next().unwrap();
        Debt {
            debtor: d.into(),
            amount: money::parse(amt).reduced(),
            creditor: c.into() }
    }

    fn parse<T>(s: &str) -> T
        where T: FromStr, T::Err: fmt::Debug {
        s.parse().unwrap()
    }

    impl Record {
        fn new(creditor: &str, amount: &str, debtors_init: &str) -> Record {
            Record {
                creditor: creditor.into(),
                amount: parse(amount),
                debtors: str2btree_set(debtors_init),
            }
        }
    }

    fn str2btree_set(xs: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        out.extend(xs.split_whitespace().map(|x| x.to_owned()));
        out
    }

    fn stringify(xs: Vec<Debt>) -> Vec<String> {
        xs.into_iter().map(|x| x.to_string()).collect()
    }
}
