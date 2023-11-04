use std::{collections::HashMap, sync::OnceLock};

use clap::Parser;
use derive_more::Display;
use itertools::Itertools;
use rand::{thread_rng, Fill, Rng};
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use strum::EnumString;

const PASSWORD_LENGTH: usize = 5;
type Problem = Password<PASSWORD_LENGTH>;
static PROBLEM_SET: OnceLock<Vec<Problem>> = OnceLock::new();

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
            .par_iter()
            .map(|(_, v)| -f64::log2(v.len() as f64 / answer_set.len() as f64))
            .sum();

        (entropy, answer_map)
    }
}

fn solve_automatically(
    problem_set: &[Problem],
    solution: Problem,
    print_steps: bool,
) -> Vec<Problem> {
    let mut answer_set = problem_set.to_vec();
    let mut answers = vec![];
    while answer_set.len() > 1 {
        let (answer, (_, mut distribution)) = problem_set
            .par_iter()
            .map(|comb| (comb.clone(), comb.calculate_entropy(&answer_set)))
            .max_by(|(_, (entropy_a, _)), (_, (entropy_b, _))| entropy_a.total_cmp(entropy_b))
            .unwrap();

        let hint = solution.check_answer(&answer);

        answer_set = distribution.remove(&hint).unwrap();

        answers.push(answer.clone());

        if print_steps {
            println!("=== {} ===", answer);
            println!("{} hits | {} remaining", hint, answer_set.len())
        }
    }

    if print_steps {
        println!("=== {} ===", answer_set.last().unwrap());
        println!("{} hits | {} remaining", 5, answer_set.len())
    }

    return answers;
}

fn assist_solving(problem_set: &[Problem]) {}

fn solve_all(problem_set: &[Problem]) {
    // do it for every possible case
    let tries = problem_set
        .clone()
        .into_par_iter()
        .enumerate()
        .map(|(i, solution)| {
            let attempts = solve_automatically(problem_set, solution.clone(), false).len();

            println!("Solved problem #{i}");
            (i, attempts)
        })
        .collect::<Vec<_>>();

    let worst_case = tries.iter().max_by(|(_, a), (_, b)| a.cmp(b)).unwrap();

    let average = tries.iter().map(|(_, b)| b).sum::<usize>() as f64 / tries.len() as f64;

    println!("Average: {average}");
    println!(
        "Worst Case: {} | {} tries",
        problem_set[worst_case.0], worst_case.1
    );
}

fn initialize_problem_set() {
    let mut problem_set = vec![
        vec![Color::Red],
        vec![Color::Green],
        vec![Color::Blue],
        vec![Color::Yellow],
    ];

    for _ in 0..PASSWORD_LENGTH - 1 {
        problem_set = problem_set
            .into_iter()
            .cartesian_product(Color::all().into_iter())
            .map(|(mut left, right)| {
                left.push(right);
                left
            })
            .collect_vec();
    }

    assert!(
        PASSWORD_LENGTH == problem_set.iter().map(Vec::len).sum::<usize>() / problem_set.len(),
        "average of length should equal length!"
    );

    let problem_set = problem_set
        .iter()
        .map(|m| Color::to_password(m))
        .collect_vec();

    let _ = PROBLEM_SET.set(problem_set);
}

#[derive(Parser, Debug)]
struct CmdArgs {
    #[arg(long)]
    all: bool,
    #[arg(long)]
    once: bool,
    #[arg(long)]
    assist: bool,
}

fn main() {
    let args = CmdArgs::parse();

    initialize_problem_set();

    let problem_set = PROBLEM_SET.get().unwrap();

    if args.all {
        println!("Solving every combination of passwords");
        solve_all(problem_set);
    }

    if args.once {
        println!("Solving one problem in detail");

        let solution: Password<PASSWORD_LENGTH> = Password::generate();
        println!("solution: {}\n", solution);

        let _ = solve_automatically(problem_set, solution, true);
    }

    // WIP
    // if args.assist {
    //     assist_solving(problem_set);
    // }
}
