use std::collections::HashMap;

use derive_more::Display;
use itertools::Itertools;
use rand::{thread_rng, Fill, Rng};
use strum::EnumString;

const PASSWORD_LENGTH: usize = 5;

type Problem = Password<PASSWORD_LENGTH>;

#[derive(Copy, Clone, PartialEq, Eq, EnumString, Display, Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    const fn index(&self) -> usize {
        match self {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
            Color::Yellow => 4,
        }
    }

    const fn abbrev(&self) -> char {
        match self {
            Color::Red => 'r',
            Color::Green => 'g',
            Color::Blue => 'b',
            Color::Yellow => 'y',
        }
    }

    const fn all() -> [Color; 4] {
        [Color::Red, Color::Green, Color::Blue, Color::Yellow]
    }

    pub fn to_password(colors: &[Color]) -> Problem {
        Password::new(colors)
    }
}

impl<const N: usize> Fill for Password<N> {
    fn try_fill<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
        let mut i = 0;
        while i < N {
            let result = rng.gen_range(0.0..=1.0);

            self.answer[i] = if (0.0..=0.25).contains(&result) {
                Color::Red
            } else if (0.25..=0.50).contains(&result) {
                Color::Green
            } else if (0.50..=0.75).contains(&result) {
                Color::Blue
            } else {
                Color::Yellow
            };
            i += 1;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Password<const N: usize> {
    answer: [Color; N],
}

impl<const N: usize> std::fmt::Display for Password<{ N }> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.answer)
    }
}

impl<const N: usize> Password<{ N }> {
    fn generate() -> Self {
        let answer = [Color::Red; N];
        let mut password = Self { answer };
        thread_rng().fill(&mut password);

        password
    }

    fn new(comb: &[Color]) -> Password<N> {
        let mut answer = [Color::Red; N];

        for i in 0..N {
            answer[i] = comb[i];
        }

        Self { answer }
    }

    fn check_answer(&self, answer: &Password<N>) -> usize {
        let mut correct = 0;

        for i in 0..N {
            if self.answer[i] == answer.answer[i] {
                correct += 1;
            }
        }

        correct
    }

    pub fn matches_description(&self, description: &Password<N>, hint: usize) -> bool {
        self.check_answer(description) == hint
    }

    pub fn calculate_entropy(
        &self,
        answer_set: &[Password<N>],
    ) -> (f64, HashMap<usize, Vec<Password<N>>>) {
        let mut answer_map = HashMap::new();

        for ans in answer_set {
            let hints = self.check_answer(ans);

            answer_map.entry(hints).or_insert(vec![]).push(ans.clone());
        }

        let entropy = answer_map
            .iter()
            .map(|(_, v)| v.len() as f64 / answer_set.len() as f64)
            .map(|probability| probability * -f64::log2(probability))
            .sum();

        (entropy, answer_map)
    }
}

fn solve_automatically(global_set: &[Problem], solution: Problem) -> usize {
    let mut answer_set = global_set.to_vec();
    let mut attempts = 0;
    while answer_set.len() > 1 {
        let mut answer = None;
        let mut best_entropy = None;
        let mut distribution = None;

        for comb in global_set.iter() {
            let entropy = comb.calculate_entropy(&answer_set);

            if best_entropy.is_none() || entropy.0 > best_entropy.unwrap() {
                best_entropy = Some(entropy.0);
                answer = Some(comb.clone());
                distribution = Some(entropy.1);
            }
        }

        let hint = solution.check_answer(answer.as_ref().unwrap());

        answer_set = distribution.unwrap().remove(&hint).unwrap();

        attempts += 1;
    }

    attempts
}

fn assist_solving(global_set: &[Problem]) {}

fn main() {
    let mut global_set = vec![
        vec![Color::Red],
        vec![Color::Green],
        vec![Color::Blue],
        vec![Color::Yellow],
    ];

    for _ in 0..PASSWORD_LENGTH - 1 {
        global_set = global_set
            .into_iter()
            .cartesian_product(Color::all().into_iter())
            .map(|(mut left, right)| {
                left.push(right);
                left
            })
            .collect_vec();
    }

    assert!(
        PASSWORD_LENGTH == global_set.iter().map(Vec::len).sum::<usize>() / global_set.len(),
        "average of length should equal length!"
    );

    let global_set = global_set
        .iter()
        .map(|m| Color::to_password(m))
        .collect_vec();

    // do it for every possible case
    let mut tries = vec![];
    let mut worst_case = None;
    let mut worst_count = 0;
    for (i, solution) in global_set.clone().into_iter().enumerate() {
        println!("Solving problem #{i}");
        let attempts = solve_automatically(&global_set, solution.clone());

        tries.push(attempts);

        if worst_case.is_none() || attempts > worst_count {
            worst_case = Some(solution);
            worst_count = attempts;
        }
    }

    let average = tries.iter().sum::<usize>() as f64 / tries.len() as f64;

    println!("Average: {average}");
    println!(
        "Worst Case: {} | {} tries",
        worst_case.unwrap(),
        worst_count
    );
}
