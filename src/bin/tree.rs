use std::collections::HashMap;
use std::rc::Rc;
use std::fs;
use std::io::Write;
use argh::FromArgs;

use wordle_pokemon::{pokemon::*, judge::*};

#[derive(Default)]
struct Node {
    guess: usize,
    rem_ans: Vec<Answer>,
    edges: HashMap<Judge,Rc<Node>>,
}

#[derive(Default)]
struct DecisionTree {
    guess_seq: Vec<Vec<Guess>>,
    judge_table: JudgeTable,
}

impl DecisionTree {

    pub fn new(filepath: &str) -> Self {
        let guess_seq: Vec<Vec<Guess>> = fs::read_to_string(filepath).unwrap().lines().map(|line| {
            line.split_whitespace().map(|s| s.parse::<Guess>().unwrap()).collect()
        }).collect();

        let n = guess_seq.len();

        Self { guess_seq, judge_table: JudgeTable::new(n) }
    }

    pub fn build(&self, rem_ans: &Vec<Answer>, depth: usize) -> Rc<Node> {
        if rem_ans.len() == 1 {
            assert!(self.guess_seq[rem_ans[0]].len() == depth + 1);
            assert!(self.guess_seq[rem_ans[0]][depth] == rem_ans[0]);
            return Rc::new(Node{ guess: rem_ans[0], rem_ans: rem_ans.clone(), ..Default::default() });
        }

        let guess = self.guess_seq[rem_ans[0]][depth];
        for i in 1..rem_ans.len() {
            assert!(self.guess_seq[rem_ans[i]].len() > depth);
            assert!(guess == self.guess_seq[rem_ans[i]][depth]);
        }

        let edges: HashMap<Judge,Rc<Node>> = self.judge_table.partition(rem_ans, &guess).iter().map(|(judge, s)| {
            (*judge, self.build(s, depth + 1))
        }).collect();

        return Rc::new(Node{ guess, edges, rem_ans: rem_ans.clone() });
    }


    pub fn next(node: &Rc<Node>, history: &Vec<Judge>) -> Rc<Node> {
        let mut v = Rc::clone(node);
        for judge in history {
            v = Rc::clone(&v.edges[judge]);
        }
        v
    }
}


#[derive(FromArgs)]
/// Build decision tree
struct Args {
    /// the number of pokemons
    #[argh(option, short='n')]
    num_pokemons: usize,

    /// the filepath of decision tree input
    #[argh(option, short='i')]
    filepath: String,
}

fn main() {
    let args: Args = argh::from_env();
    let n = args.num_pokemons;

    let pokemons = PokemonList::new(n);

    let root = DecisionTree::new(&args.filepath).build(&pokemons.all_ans, 0);

    let mut history = vec![];

    loop {
        let nxt = DecisionTree::next(&root, &history);
        println!("(残り{}匹) {}", nxt.rem_ans.len(), POKEMONS[nxt.guess]);

        print!("-> ");
        std::io::stdout().flush().unwrap();
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).unwrap();
        let s = s.trim().to_string();
        if s.len() != 5 {
            println!("s.len(): {}", s.len());
        }

        assert!(s.len() == 5);
        history.push({
            let judge = s.chars().enumerate().map(|(i, c)| {
                c.to_digit(10).unwrap() << 2*i
            }).sum::<u32>() as Judge;
            if judge == ALL_CORRECT {
                println!("Congratulations!!!");
                break;
            }
            judge
        });
    }
}